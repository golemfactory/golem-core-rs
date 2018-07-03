use std::{error, net};

use actix::io::{FramedWrite, WriteHandler};
use actix::prelude::*;
use actix::Unsync;
use futures::stream::Stream;
use futures::{future, Future};
use tokio_codec::FramedRead;
use tokio_io::io::WriteHalf;
use tokio_io::AsyncRead;
use tokio_tcp::{TcpListener, TcpStream};

use codec::error::CodecError;
use codec::message::Message;
use codec::MessageCodec;
use network::*;
use transport::error::*;
use transport::message::*;
use transport::*;

pub type TcpActorAddr<N> = Addr<Unsync, TcpTransport<N>>;
pub type TcpSessionAddr<N> = Addr<Unsync, TcpSession<N>>;

/// Session creation message (TCP exclusive)
#[derive(Debug, Message)]
struct CreateSession {
    stream: TcpStream,
    initiator: bool,
}

/// TCP transport actor
pub struct TcpTransport<N>
where
    N: Network + 'static,
    N::Context: AsyncContext<N>,
{
    /// Socket address
    pub address: net::SocketAddr,
    /// Network actor address
    pub network: NetAddr<N>,
    /// Own actor address
    pub actor: TcpActorAddr<N>,
}

impl<N> TcpTransport<N>
where
    N: Network + 'static,
    N::Context: AsyncContext<N>,
{
    pub fn run(
        network: NetAddr<N>,
        address: net::SocketAddr,
    ) -> Result<TcpActorAddr<N>, Box<error::Error>> {
        let listener = match TcpListener::bind(&address) {
            Ok(l) => l,
            Err(_) => return Err(Box::new(cannot_listen_error("TCP", &address))),
        };

        // store the actual IP address and port
        let address = listener.local_addr()?;

        let router = TcpTransport::create(move |ctx| {
            let flow = listener
                .incoming()
                .map_err(|_| ())
                .map(move |stream| CreateSession {
                    stream,
                    initiator: false,
                });

            ctx.add_message_stream(flow);
            TcpTransport {
                address,
                network,
                actor: ctx.address(),
            }
        });

        Ok(router)
    }
}

impl<N> Actor for TcpTransport<N>
where
    N: Network + 'static,
    N::Context: AsyncContext<N>,
{
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        let actor = Transport::Tcp(self.actor.clone());
        let address = self.address.clone();
        let msg = Listening{ actor, address };

        let future = self
            .network
            .send(msg)
            .map_err(|_| eprintln!("TCP -> network error (started)"));

        Arbiter::handle().spawn(future);
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        let actor = Transport::Tcp(self.actor.clone());
        let address = self.address.clone();
        let msg = Stopped{ actor, address };

        let future = self
            .network
            .send(msg)
            .map_err(|_| eprintln!("TCP -> network error (stopping)"));

        Arbiter::handle().spawn(future);
        Running::Stop
    }
}

//
// Message handlers
//

impl<N> Handler<CreateSession> for TcpTransport<N>
where
    N: Network + 'static,
    N::Context: AsyncContext<N>,
{
    type Result = NoResult;

    fn handle(&mut self, msg: CreateSession, _: &mut Context<Self>) {
        let address = msg.stream.peer_addr().unwrap();
        let initiator = false;

        TcpSession::<N>::run(self.network.clone(), address, msg.stream, initiator);
    }
}

impl<N> Handler<Connect> for TcpTransport<N>
where
    N: Network + 'static,
    N::Context: AsyncContext<N>,
{
    type Result = EmptyResult;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        let network = self.network.clone();
        let future = TcpStream::connect(&msg.address)
            .map_err(move |e| {
                eprintln!("Connect error {}", e);
            })
            .and_then(move |stream| {
                let address = stream.peer_addr().unwrap();
                let initiator = true;

                TcpSession::run(network, address, stream, initiator);

                future::ok(())
            });

        Arbiter::handle().spawn(future);
        Ok(())
    }
}

impl<N> Handler<Stop> for TcpTransport<N>
where
    N: Network + 'static,
    N::Context: AsyncContext<N>,
{
    type Result = EmptyResult;

    fn handle(&mut self, _: Stop, ctx: &mut Context<Self>) -> Self::Result {
        ctx.stop();
        Ok(())
    }
}

/// TCP session actor
pub struct TcpSession<N>
where
    N: Network + 'static,
    N::Context: AsyncContext<N>,
{
    /// Session network
    network: NetAddr<N>,
    /// Remote address
    address: net::SocketAddr,
    /// Framed writer
    writer: FramedWrite<WriteHalf<TcpStream>, MessageCodec>,
    /// Own actor address
    actor: TcpSessionAddr<N>,
    /// Whether session was initiated by us
    initiator: bool,
}

impl<N> TcpSession<N>
where
    N: Network + 'static,
    N::Context: AsyncContext<N>,
{
    fn run(
        network: NetAddr<N>,
        address: net::SocketAddr,
        stream: TcpStream,
        initiator: bool,
    ) -> TcpSessionAddr<N> {
        TcpSession::create(move |ctx| {
            let (read, write) = stream.split();
            let reader = FramedRead::new(read, MessageCodec);
            let writer = FramedWrite::new(write, MessageCodec, ctx);

            TcpSession::add_stream(reader, ctx);
            TcpSession {
                network,
                address,
                writer,
                actor: ctx.address(),
                initiator,
            }
        })
    }
}

impl<N> WriteHandler<CodecError> for TcpSession<N>
where
    N: Network + 'static,
    N::Context: AsyncContext<N>,
{}

impl<N> Actor for TcpSession<N>
where
    N: Network + 'static,
    N::Context: AsyncContext<N>,
{
    type Context = Context<Self>;

    fn started(&mut self, _: &mut <Self as Actor>::Context) {
        let session = self.actor.clone();
        let msg = Connected {
            transport: TransportProtocol::Tcp,
            address: self.address.clone(),
            session: TransportSession::Tcp(session),
            initiator: self.initiator,
        };

        let future = self.network.send(msg).map_err(|_| {});

        Arbiter::handle().spawn(future);
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        let msg = Disconnected {
            transport: TransportProtocol::Tcp,
            address: self.address.clone(),
        };

        let future = self.network.send(msg).map_err(|_| {});

        Arbiter::handle().spawn(future);
        Running::Stop
    }
}

//
// Message handlers
//

impl<N> StreamHandler<Message, CodecError> for TcpSession<N>
where
    N: Network + 'static,
    N::Context: AsyncContext<N>,
{
    fn handle(&mut self, msg: Message, _ctx: &mut Self::Context) {
        let msg = ReceivedMessage {
            transport: TransportProtocol::Tcp,
            address: self.address.clone(),
            message: msg,
        };

        let future = self.network.send(msg).map_err(|_| {});

        Arbiter::handle().spawn(future);
    }
}

impl<N> Handler<SessionSendMessage> for TcpSession<N>
where
    N: Network + 'static,
    N::Context: AsyncContext<N>,
{
    type Result = NoResult;

    fn handle(&mut self, msg: SessionSendMessage, _ctx: &mut Self::Context) {
        if self.writer.closed() {
            eprintln!("Trying to write to a closed TCP stream ({})", self.address);
            return;
        }

        self.writer.write(msg.message);
    }
}

impl<N> Handler<Stop> for TcpSession<N>
where
    N: Network + 'static,
    N::Context: AsyncContext<N>,
{
    type Result = EmptyResult;

    fn handle(&mut self, _: Stop, ctx: &mut Self::Context) -> Self::Result {
        ctx.stop();
        Ok(())
    }
}
