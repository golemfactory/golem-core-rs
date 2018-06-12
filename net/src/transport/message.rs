use std::{io, net};

use actix;
use futures::Future;

use codec::message::Message;
use logic::*;
use transport::*;

pub type EmptyResponse = ();
pub type FutureResponse = Result<
    Box<Future<Item=(), Error=io::Error>>,
    actix::MailboxError
>;

//
// Requests
//

#[derive(Message, Debug)]
#[rtype(result="FutureResponse")]
pub struct SendMessage
{
    pub transport:  Transport,
    pub address:    net::SocketAddr,
    pub message:    Message,
}

#[derive(Message, Debug)]
#[rtype(result="EmptyResponse")]
pub struct SessionSendMessage
{
    pub address:    net::SocketAddr,
    pub message:    Message,
}

#[derive(Message, Debug)]
#[rtype(result="FutureResponse")]
pub struct Connect
{
    pub transport:  Transport,
    pub address:    net::SocketAddr,
}

#[derive(Message, Debug)]
#[rtype(result="FutureResponse")]
pub struct Disconnect
{
    pub transport:  Transport,
    pub address:    net::SocketAddr,
}

#[derive(Message, Debug)]
#[rtype(result="FutureResponse")]
pub struct Stop(pub Transport);

//
// Events
//

#[derive(Message, Debug)]
#[rtype(result="EmptyResponse")]
pub struct Received
{
    pub transport:  Transport,
    pub address:    net::SocketAddr,
    pub message:    Message,
}

#[derive(Message)]
#[rtype(result="EmptyResponse")]
pub struct Listening<L>(pub TransportRouter<L>)
where
    L: Logic + 'static,
    L::Context: actix::AsyncContext<L>,
;

#[derive(Message)]
#[rtype(result="EmptyResponse")]
pub struct Stopped<L>(pub TransportRouter<L>)
where
    L: Logic + 'static,
    L::Context: actix::AsyncContext<L>,
;

#[derive(Message)]
#[rtype(result="EmptyResponse")]
pub struct Connected<L>
where
    L: Logic + 'static,
    L::Context: actix::AsyncContext<L>,
{
    pub transport:  Transport,
    pub address:    net::SocketAddr,
    pub session:    TransportSession<L>,
    pub initiator:  bool,
}

#[derive(Message, Debug)]
#[rtype(result="EmptyResponse")]
pub struct Disconnected
{
    pub transport:  Transport,
    pub address:    net::SocketAddr,
}
