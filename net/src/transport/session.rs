use std::net;
use std::collections::HashMap;

use actix;

use transport::{Transport, TransportSession};
use logic::Logic;

pub struct Sessions<L>(HashMap<(Transport, net::SocketAddr), TransportSession<L>>)
where
    L: Logic + 'static,
    L::Context: actix::AsyncContext<L>,
;

impl<L> Sessions<L>
where
    L: Logic + 'static,
    L::Context: actix::AsyncContext<L>,
{
    pub fn new()
        -> Self
    {
        Sessions(HashMap::new())
    }

    pub fn get(&self, transport: &Transport, address: &net::SocketAddr)
        -> Option<&TransportSession<L>>
    {
        let key = (*transport, *address);
        let key = &key;
        self.0.get(key)
    }

    pub fn add(
        &mut self,
        transport: Transport,
        address: net::SocketAddr,
        session: TransportSession<L>
    )   -> ()
    {
        self.0.insert((transport, address), session);
    }

    pub fn remove(&mut self, transport: &Transport, address: &net::SocketAddr)
        -> Option<TransportSession<L>>
    {
        let key = (*transport, *address);
        let key = &key;
        self.0.remove(key)
    }
}
