use std::{error, net};

use actix::io::{FramedWrite, WriteHandler};
use actix::prelude::*;
use futures::{future, Future};
use futures::stream::Stream;
use tokio_io::AsyncRead;
use tokio_io::io::WriteHalf;
use tokio_io::codec::FramedRead;
use tokio_tcp::{TcpListener, TcpStream};

use codec::MessageCodec;
use codec::error::CodecError;
use codec::message::Message;
use logic::*;
use transport::*;
use transport::message::*;

pub type TcpTransportAddr<L> = Addr<AddrType, TcpTransport<L>>;
pub type TcpSessionAddr<L> = Addr<AddrType, TcpSession<L>>;


#[derive(Debug, Message)]
struct CreateSession
{
    stream: TcpStream,
    initiator: bool,
}

//
// TCP transport
//

pub struct TcpTransport<L>
where
    L: Logic + 'static,
    L::Context: AsyncContext<L>,
{
    pub address: net::SocketAddr,
    pub logic: LogicAddr<L>,
    pub actor: TcpTransportAddr<L>,
}

impl<L> TcpTransport<L>
where
    L: Logic + 'static,
    L::Context: AsyncContext<L>,
{
    pub fn run(logic: LogicAddr<L>, address: net::SocketAddr)
        -> Result<TcpTransportAddr<L>, Box<error::Error>>
    {
        let listener = match TcpListener::bind(&address) {
            Ok(l) => l,
            Err(_) => return Err(Box::new(mailbox_io_error())),
        };

        let router = TcpTransport::create(move |ctx| {
            let flow = listener.incoming()
                .map_err(|_| ())
                .map(move |stream| CreateSession{ stream, initiator: false });

            ctx.add_message_stream(flow);
            TcpTransport{ address, logic, actor: ctx.address() }
        });

        Ok(router)
    }
}

impl<L> Actor for TcpTransport<L>
where
    L: Logic + 'static,
    L::Context: AsyncContext<L>,
{
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context)
    {
        let router = TransportRouter::Tcp(self.actor.clone());
        let msg = Listening(router);

        let future = self.logic
            .send(msg)
            .map_err(|_| eprintln!("TCP -> logic error (started)"));

        Arbiter::handle().spawn(future);
    }

    fn stopping(&mut self, _ctx: &mut Self::Context)
        -> Running
    {
        let router = TransportRouter::Tcp(self.actor.clone());
        let msg = Stopped(router);

        let future = self.logic
            .send(msg)
            .map_err(|_| eprintln!("TCP -> logic error (stopping)"));

        Arbiter::handle().spawn(future);
        Running::Stop
    }
}

//
// Message handlers
//

impl<L> Handler<CreateSession> for TcpTransport<L>
where
    L: Logic + 'static,
    L::Context: AsyncContext<L>,
{
    type Result = EmptyResponse;

    fn handle(&mut self, msg: CreateSession, _: &mut Context<Self>)
    {
        let address = msg.stream.peer_addr().unwrap();
        let initiator = false;

        TcpSession::<L>::run(
            self.logic.clone(),
            address,
            msg.stream,
            initiator
        );
    }
}

impl<L> Handler<Connect> for TcpTransport<L>
where
    L: Logic + 'static,
    L::Context: AsyncContext<L>,
{
    type Result = FutureResponse;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>)
        -> Self::Result
    {
        let logic = self.logic.clone();
        let result = future::ok(());

        let future = TcpStream::connect(&msg.address)
            .map_err(move |e| {
                eprintln!("Connect error {}", e);
            })
            .and_then(move |stream| {
                let address = stream.peer_addr().unwrap();
                let initiator = true;

                TcpSession::run(
                    logic,
                    address,
                    stream,
                    initiator
                );

                future::ok(())
            });

        Arbiter::handle().spawn(future);
        Ok(Box::new(result))
    }
}

impl<L> Handler<Stop> for TcpTransport<L>
where
    L: Logic + 'static,
    L::Context: AsyncContext<L>,
{
    type Result = FutureResponse;

    fn handle(&mut self, _: Stop, ctx: &mut Context<Self>)
        -> FutureResponse
    {
        ctx.stop();

        let future = future::ok(());
        Ok(Box::new(future))
    }
}

//
// TCP session
//

pub struct TcpSession<L>
where
    L: Logic + 'static,
    L::Context: AsyncContext<L>,
{
    /// Session logic
    logic: LogicAddr<L>,
    /// Remote address
    address: net::SocketAddr,
    /// Framed writer
    writer: FramedWrite<WriteHalf<TcpStream>, MessageCodec>,
    /// Own actor address
    actor: TcpSessionAddr<L>,
    /// Whether session was initiated by us
    initiator: bool,
}

impl<L> TcpSession<L>
where
    L: Logic + 'static,
    L::Context: AsyncContext<L>,
{
    fn run(
        logic: LogicAddr<L>,
        address: net::SocketAddr,
        stream: TcpStream,
        initiator: bool,
    )   -> TcpSessionAddr<L>
    {
        TcpSession::create(move |ctx| {
            let (read, write) = stream.split();
            let reader = FramedRead::new(read, MessageCodec);
            let writer = FramedWrite::new(write, MessageCodec, ctx);

            TcpSession::add_stream(reader, ctx);
            TcpSession{
                logic,
                address,
                writer,
                actor: ctx.address(),
                initiator
            }
        })
    }
}

impl<L> WriteHandler<CodecError> for TcpSession<L>
where
    L: Logic + 'static,
    L::Context: AsyncContext<L>,
{}

//
// TCP session actor
//

impl<L> Actor for TcpSession<L>
where
    L: Logic + 'static,
    L::Context: AsyncContext<L>,
{
    type Context = Context<Self>;

    fn started(&mut self, _: &mut <Self as Actor>::Context)
    {
        let session = self.actor.clone();
        let msg = Connected {
            transport: Transport::Tcp,
            address: self.address.clone(),
            session: TransportSession::Tcp(session),
            initiator: self.initiator
        };

        let future = self.logic
            .send(msg)
            .map_err(|_| {});

        Arbiter::handle().spawn(future);
    }

    fn stopping(&mut self, _ctx: &mut Self::Context)
        -> Running
    {
        let msg = Disconnected {
            transport: Transport::Tcp,
            address: self.address.clone(),
        };

        let future = self.logic
            .send(msg)
            .map_err(|_| {});

        Arbiter::handle().spawn(future);
        Running::Stop
    }
}

//
// TCP session message handlers
//

impl<L> StreamHandler<Message, CodecError> for TcpSession<L>
where
    L: Logic + 'static,
    L::Context: AsyncContext<L>,
{
    fn handle(&mut self, msg: Message, _ctx: &mut Self::Context)
    {
        let msg = Received{
            transport: Transport::Tcp,
            address: self.address.clone(),
            message: msg,
        };

        let future = self.logic
            .send(msg)
            .map_err(|_| {});

        Arbiter::handle().spawn(future);
    }
}

impl<L> Handler<SessionSendMessage> for TcpSession<L>
where
    L: Logic + 'static,
    L::Context: AsyncContext<L>,
{
    type Result = EmptyResponse;

    fn handle(&mut self, msg: SessionSendMessage, _ctx: &mut Self::Context)
    {
        if self.writer.closed() {
            eprintln!("Trying to write to a closed TCP stream ({})", self.address);
            return;
        }

        self.writer.write(msg.message);
    }
}

impl<L> Handler<Stop> for TcpSession<L>
where
    L: Logic + 'static,
    L::Context: AsyncContext<L>,
{
    type Result = FutureResponse;

    fn handle(&mut self, _: Stop, ctx: &mut Self::Context)
        -> FutureResponse
    {
        ctx.stop();

        let result = future::ok(());
        Ok(Box::new(result))
    }
}
