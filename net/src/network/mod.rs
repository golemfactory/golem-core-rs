pub mod session;

use actix::{Actor, Addr, AsyncContext, Handler, Unsync};
use transport::message::*;


pub type NetAddr<N> = Addr<Unsync, N>;

pub trait Network:
    Actor
    + Handler<Listening<Self>>
    + Handler<ReceivedMessage>
    + Handler<SendMessage>
    + Handler<Stop>
    + Handler<Stopped<Self>>
    + Handler<Connect>
    + Handler<Connected<Self>>
    + Handler<Disconnect>
    + Handler<Disconnected>
where
    Self::Context: AsyncContext<Self>,
{
}
