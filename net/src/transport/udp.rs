use std::{error, net};

use actix::io::WriteHandler;
use actix::prelude::*;
use actix::Unsync;
use futures::sink::SinkFromErr;
use futures::stream::{MapErr, SplitSink, Stream};
use futures::sync::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::{Future, Sink};

use tokio_udp::{UdpFramed, UdpSocket};

use codec::error::CodecError;
use codec::message::Message;
use codec::MessageCodec;
use network::*;
use transport::message::*;
use transport::*;

pub type UdpActorAddr<N> = Addr<Unsync, UdpTransport<N>>;

type FromErrType = SinkFromErr<SplitSink<UdpFramed<MessageCodec>>, CodecError>;
type ReceiverType = UnboundedReceiver<(Message, net::SocketAddr)>;

// Actor messages
#[derive(Message)]
pub struct UdpPacket {
    pub address: net::SocketAddr,
    pub message: Message,
}

pub struct UdpTransport<N>
where
    N: Network + 'static,
    N::Context: AsyncContext<N>,
{
    address: net::SocketAddr,
    logic: NetAddr<N>,
    sender: UnboundedSender<(Message, net::SocketAddr)>,
    actor: UdpActorAddr<N>,
}

impl<N> UdpTransport<N>
where
    N: Network + 'static,
    N::Context: AsyncContext<N>,
{
    pub fn run(
        logic: NetAddr<N>,
        address: net::SocketAddr,
    ) -> Result<UdpActorAddr<N>, Box<error::Error>> {
        let socket = match UdpSocket::bind(&address) {
            Ok(s) => s,
            Err(e) => return Err(e.into()),
        };

        let (sink, stream) = UdpFramed::new(socket, MessageCodec {}).split();
        let (sender, receiver) = unbounded();

        let router = UdpTransport::create(move |ctx| {
            let map_fn = |_: (FromErrType, MapErr<ReceiverType, _>)| ();

            // ingress stream
            ctx.add_stream(stream.map(|(m, a)| UdpPacket {
                address: a,
                message: m,
            }));

            // egress stream
            Arbiter::handle().spawn(
                sink.sink_from_err()
                    .send_all(receiver.map_err(move |e| {
                        eprintln!("UDP receiver channel error: {:?}", e);
                    }))
                    .map(map_fn)
                    .map_err(move |e| {
                        eprintln!("UDP send error: {}", e);
                    }),
            );

            UdpTransport {
                address,
                logic,
                sender,
                actor: ctx.address(),
            }
        });

        Ok(router)
    }
}

impl<N> Handler<Stop> for UdpTransport<N>
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

impl<N> Handler<SessionSendMessage> for UdpTransport<N>
where
    N: Network + 'static,
    N::Context: AsyncContext<N>,
{
    type Result = NoResult;

    fn handle(&mut self, msg: SessionSendMessage, _ctx: &mut Self::Context) {
        let data = (msg.message, msg.address);
        if let Err(e) = self.sender.unbounded_send(data) {
            eprintln!("UDP unbounded send failed: {}", e);
        }
    }
}

impl<N> WriteHandler<CodecError> for UdpTransport<N>
where
    N: Network + 'static,
    N::Context: AsyncContext<N>,
{}

impl<N> StreamHandler<UdpPacket, CodecError> for UdpTransport<N>
where
    N: Network + 'static,
    N::Context: AsyncContext<N>,
{
    fn handle(&mut self, pkt: UdpPacket, _: &mut Context<Self>) {
        let msg = ReceivedMessage {
            transport: TransportProtocol::Udp,
            address: self.address.clone(),
            message: pkt.message,
        };

        let future = self.logic.send(msg).map_err(|_| {});

        Arbiter::handle().spawn(future);
    }
}

impl<N> Actor for UdpTransport<N>
where
    N: Network + 'static,
    N::Context: AsyncContext<N>,
{
    type Context = Context<Self>;

    fn started(&mut self, _: &mut <Self as Actor>::Context) {
        let actor = self.actor.clone();
        let msg = Listening(Transport::Udp(actor));

        let future = self.logic.send(msg).map_err(|_| {});

        Arbiter::handle().spawn(future);
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        let actor = self.actor.clone();
        let msg = Stopped(Transport::Udp(actor));

        let future = self.logic.send(msg).map_err(|_| {});

        Arbiter::handle().spawn(future);
        Running::Stop
    }
}
