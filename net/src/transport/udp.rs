use std::{error, net};

use actix::io::WriteHandler;
use actix::prelude::*;
use futures::{future, Future, Sink};
use futures::sink::SinkFromErr;
use futures::stream::{MapErr, SplitSink, Stream};
use futures::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};

use tokio_udp::{UdpFramed, UdpSocket};

use codec::MessageCodec;
use codec::error::CodecError;
use codec::message::Message;
use logic::*;
use transport::*;
use transport::message::*;


pub type UdpTransportAddr<L> = Addr<AddrType, UdpTransport<L>>;

type FromErrType = SinkFromErr<SplitSink<UdpFramed<MessageCodec>>, CodecError>;
type ReceiverType = UnboundedReceiver<(Message, net::SocketAddr)>;

// Actor messages
#[derive(Message)]
pub struct UdpPacket
{
    pub address: net::SocketAddr,
    pub message: Message,
}

pub struct UdpTransport<L>
where
    L: Logic + 'static,
    L::Context: AsyncContext<L>,
{
    address: net::SocketAddr,
    logic: LogicAddr<L>,
    sender: UnboundedSender<(Message, net::SocketAddr)>,
    actor: UdpTransportAddr<L>,
}

impl<L> UdpTransport<L>
where
    L: Logic + 'static,
    L::Context: AsyncContext<L>,
{
    pub fn run(logic: LogicAddr<L>, address: net::SocketAddr)
        -> Result<UdpTransportAddr<L>, Box<error::Error>>
    {
        let socket = match UdpSocket::bind(&address) {
            Ok(l) => l,
            Err(e) => return Err(e.into()),
        };

        let (sink, stream) = UdpFramed::new(socket, MessageCodec{}).split();
        let (sender, receiver) = unbounded();

        let router = UdpTransport::create(move |ctx| {

            let map_fn = |_: (FromErrType, MapErr<ReceiverType, _>)| ();

            // ingress stream
            ctx.add_stream(stream.map(|(m, a)| UdpPacket{ address: a, message: m }));

            // egress stream
            Arbiter::handle().spawn(
                sink.sink_from_err()
                    .send_all(
                        receiver.map_err(move |e| {
                            eprintln!("UDP receiver channel error: {:?}", e);
                        })
                    )
                    .map(map_fn)
                    .map_err(move |e| {
                        eprintln!("UDP send error: {}", e);
                    })
            );

            UdpTransport { address, logic, sender, actor: ctx.address() }
        });

        Ok(router)
    }
}

impl<L> Handler<Stop> for UdpTransport<L>
    where
        L: Logic + 'static,
        L::Context: AsyncContext<L>,
{
    type Result = FutureResponse;

    fn handle(&mut self, _: Stop, ctx: &mut Self::Context)
        -> FutureResponse
    {
        ctx.stop();
        let future = future::ok(());
        Ok(Box::new(future))
    }
}

impl<L> Handler<SessionSendMessage> for UdpTransport<L>
where
    L: Logic + 'static,
    L::Context: AsyncContext<L>,
{
    type Result = EmptyResponse;

    fn handle(&mut self, msg: SessionSendMessage, _ctx: &mut Self::Context)
    {
        let data = (msg.message, msg.address);
        if let Err(e) = self.sender.unbounded_send(data) {
            eprintln!("UDP unbounded send failed: {}", e);
        }
    }
}

impl<L> WriteHandler<CodecError> for UdpTransport<L>
where
    L: Logic + 'static,
    L::Context: AsyncContext<L>,
{}

impl<L> StreamHandler<UdpPacket, CodecError> for UdpTransport<L>
where
    L: Logic + 'static,
    L::Context: AsyncContext<L>,
{
    fn handle(&mut self, pkt: UdpPacket, _: &mut Context<Self>)
    {
        let msg = Received{
            transport: Transport::Udp,
            address: self.address.clone(),
            message: pkt.message,
        };

        let future = self.logic
            .send(msg)
            .map_err(|_| {});

        Arbiter::handle().spawn(future);
    }
}

impl<L> Actor for UdpTransport<L>
where
    L: Logic + 'static,
    L::Context: AsyncContext<L>,
{
    type Context = Context<Self>;

    fn started(&mut self, _: &mut <Self as Actor>::Context)
    {
        let actor = self.actor.clone();
        let msg = Listening(TransportRouter::Udp(actor));

        let future = self.logic
            .send(msg)
            .map_err(|_| {});

        Arbiter::handle().spawn(future);
    }

    fn stopping(&mut self, _ctx: &mut Self::Context)
                -> Running
    {
        let actor = self.actor.clone();
        let msg = Stopped(TransportRouter::Udp(actor));

        let future = self.logic
            .send(msg)
            .map_err(|_| {});

        Arbiter::handle().spawn(future);
        Running::Stop
    }
}
