use std::{io, net};

use actix;
use futures::Future;

use codec::message::Message;
use network::*;
use transport::*;

pub type NoResult = ();
pub type EmptyResult = Result<(), actix::MailboxError>;

pub type BoxedFuture = Box<Future<Item = (), Error = io::Error>>;
pub type FutureResult = Result<BoxedFuture, actix::MailboxError>;

//
// Requests
//

#[derive(Message, Debug)]
#[rtype(result = "EmptyResult")]
pub struct SendMessage {
    pub transport: TransportProtocol,
    pub address: net::SocketAddr,
    pub message: Message,
}

unsafe impl Send for SendMessage {}

#[derive(Message, Debug)]
#[rtype(result = "NoResult")]
pub struct SessionSendMessage {
    pub address: net::SocketAddr,
    pub message: Message,
}

unsafe impl Send for SessionSendMessage {}

#[derive(Message, Debug)]
#[rtype(result = "EmptyResult")]
pub struct Connect {
    pub transport: TransportProtocol,
    pub address: net::SocketAddr,
}

unsafe impl Send for Connect {}

#[derive(Message, Debug)]
#[rtype(result = "EmptyResult")]
pub struct Disconnect {
    pub transport: TransportProtocol,
    pub address: net::SocketAddr,
}

unsafe impl Send for Disconnect {}

#[derive(Message, Clone, Debug)]
#[rtype(result = "EmptyResult")]
pub struct Stop(pub TransportProtocol);

unsafe impl Send for Stop {}

//
// Events
//

#[derive(Message, Debug)]
#[rtype(result = "NoResult")]
pub struct ReceivedMessage {
    pub transport: TransportProtocol,
    pub address: net::SocketAddr,
    pub message: Message,
}

unsafe impl Send for ReceivedMessage {}

#[derive(Message)]
#[rtype(result = "NoResult")]
pub struct Listening<N>
where
    N: Network + 'static,
    N::Context: actix::AsyncContext<N>
{
    pub actor: Transport<N>,
    pub address: net::SocketAddr,
}

unsafe impl<N> Send for Listening<N>
where
    N: Network + 'static,
    N::Context: actix::AsyncContext<N>,
{}

#[derive(Message)]
#[rtype(result = "NoResult")]
pub struct Stopped<N>
where
    N: Network + 'static,
    N::Context: actix::AsyncContext<N>
{
    pub actor: Transport<N>,
    pub address: net::SocketAddr,
}

unsafe impl<N> Send for Stopped<N>
where
    N: Network + 'static,
    N::Context: actix::AsyncContext<N>,
{}

#[derive(Message)]
#[rtype(result = "NoResult")]
pub struct Connected<N>
where
    N: Network + 'static,
    N::Context: actix::AsyncContext<N>,
{
    pub transport: TransportProtocol,
    pub address: net::SocketAddr,
    pub session: TransportSession<N>,
    pub initiator: bool,
}

unsafe impl<N> Send for Connected<N>
where
    N: Network + 'static,
    N::Context: actix::AsyncContext<N>,
{}

#[derive(Message, Debug)]
#[rtype(result = "NoResult")]
pub struct Disconnected {
    pub transport: TransportProtocol,
    pub address: net::SocketAddr,
}

unsafe impl Send for Disconnected {}
