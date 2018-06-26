use std::sync::mpsc;
use std::thread;

use actix::prelude::*;
use actix::Syn;
use cpython::*;
use futures::sync::mpsc::{unbounded, UnboundedReceiver};
use futures::{Future, Stream};

use net::codec::message::{Encapsulated, Message};
use net::transport::message::*;
use net::transport::tcp::TcpTransport;
use net::transport::udp::UdpTransport;
use net::transport::*;

use error::ModuleError;
use event::CoreEvent;
use network::*;
use python::*;

pub struct Core {
    pub network: Option<Addr<Syn, NetworkCore>>,
    pub rx: Option<UnboundedReceiver<CoreEvent>>,
}

impl Core {
    pub fn run(&mut self, py_host: PyString, py_port: PyLong) -> Result<(), ModuleError> {
        // convert to socket address
        let address = to_socket_address(py_host, py_port)?;
        // start callback channel
        let (tx, rx) = unbounded();
        // start initialization channel
        let (tx_init, rx_init) = mpsc::channel();

        // spawn the network thread
        thread::spawn(move || {
            let sys = System::new("net");
            let (unsync, syn) = NetworkCore::run(tx);

            if let Err(e) = TcpTransport::run(unsync.clone(), address.clone()) {
                let e = ModuleError::from(e);
                tx_init.send(Err(e)).ok();
                return;
            }

            if let Err(e) = UdpTransport::run(unsync.clone(), address.clone()) {
                let e = ModuleError::from(e);
                tx_init.send(Err(e)).ok();
                return;
            }

            if let Err(_) = tx_init.send(Ok(syn)) {
                return;
            }

            sys.run();
        });

        // receive and return the (syn) logic address
        let result = rx_init.recv()?;
        match result {
            Ok(a) => {
                self.network = Some(a);
                self.rx = Some(rx);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub fn run_rx_queue(&mut self, queue: PyShared) -> bool {
        if let None = self.rx {
            return false;
        }

        let rx = self.rx.take().unwrap();
        let reader =
            rx.for_each(move |event| {
                let args: PyTuple = event.into();
                py_call_method!(queue, "put", args);
                Ok(())
            }).map_err(|_| eprintln!("Core event receiver error"));

        Arbiter::handle().spawn(reader);

        true
    }

    pub fn running(&self) -> bool {
        match self.network {
            Some(_) => true,
            None => false,
        }
    }
}

impl Core {
    pub fn connect(
        &self,
        py_protocol: PyLong,
        py_host: PyString,
        py_port: PyLong,
    ) -> Result<(), ModuleError> {
        let address = to_socket_address(py_host, py_port)?;
        let protocol = py_extract!(py_protocol, u16)?;
        let transport = TransportProtocol::from(protocol);

        self.network_send(Connect { transport, address })
            .or_else(|e| Err(ModuleError::from(e)))
    }

    pub fn disconnect(
        &self,
        py_protocol: PyLong,
        py_host: PyString,
        py_port: PyLong,
    ) -> Result<(), ModuleError> {
        let address = to_socket_address(py_host, py_port)?;
        let protocol = py_extract!(py_protocol, u16)?;
        let transport = TransportProtocol::from(protocol);

        self.network_send(Disconnect { transport, address })
            .or_else(|e| Err(ModuleError::from(e)))
    }

    pub fn send(
        &self,
        py_protocol: PyLong,
        py_host: PyString,
        py_port: PyLong,
        py_protocol_id: PyLong,
        message: Vec<u8>,
    ) -> Result<(), ModuleError> {
        let address = to_socket_address(py_host, py_port)?;
        let protocol = py_extract!(py_protocol, u16)?;
        let protocol_id = py_extract!(py_protocol_id, u16)?;

        let msg = SendMessage {
            transport: TransportProtocol::from(protocol),
            address,
            message: Message::Encapsulated(Encapsulated {
                protocol_id,
                message,
            }),
        };

        self.network_send(msg)
            .or_else(|e| Err(ModuleError::from(e)))
    }
}

impl Core {
    fn network_send<M>(&self, msg: M) -> Result<(), MailboxError>
    where
        M: actix::Message + Send + 'static,
        M::Result: Send,
        NetworkCore: actix::Handler<M>,
    {
        match self.network {
            Some(ref n) => {
                let future = n.send(msg).map(|_| ()).map_err(|_| {});
                Arbiter::handle().spawn(future);
                Ok(())
            }
            None => Err(MailboxError::Closed),
        }
    }
}
