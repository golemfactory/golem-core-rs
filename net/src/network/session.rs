use actix;
use std::collections::HashMap;
use std::net;

use network::Network;
use transport::{TransportProtocol, TransportSession};

pub struct Sessions<N>(HashMap<(TransportProtocol, net::SocketAddr), TransportSession<N>>)
where
    N: Network + 'static,
    N::Context: actix::AsyncContext<N>;

impl<N> Sessions<N>
where
    N: Network + 'static,
    N::Context: actix::AsyncContext<N>,
{
    pub fn new() -> Self {
        Sessions(HashMap::new())
    }

    pub fn get(
        &self,
        protocol: &TransportProtocol,
        address: &net::SocketAddr,
    ) -> Option<&TransportSession<N>> {
        let key = (*protocol, *address);
        self.0.get(&key)
    }

    pub fn add(
        &mut self,
        transport: TransportProtocol,
        address: net::SocketAddr,
        session: TransportSession<N>,
    ) -> () {
        self.0.insert((transport, address), session);
    }

    pub fn remove(
        &mut self,
        protocol: &TransportProtocol,
        address: &net::SocketAddr,
    ) -> Option<TransportSession<N>> {
        let key = (*protocol, *address);
        self.0.remove(&key)
    }
}
