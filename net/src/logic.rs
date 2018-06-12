use actix::{Actor, Addr, AsyncContext, Handler};

use transport::*;
use transport::message::*;


pub type LogicAddr<L> = Addr<AddrType, L>;

pub trait Logic
    :   Actor +
        Handler<Listening<Self>> +
        Handler<Received> +
        Handler<SendMessage> +
        Handler<Stop> +
        Handler<Stopped<Self>> +
        Handler<Connect> +
        Handler<Connected<Self>> +
        Handler<Disconnect> +
        Handler<Disconnected>
where
    Self::Context: AsyncContext<Self>
{}
