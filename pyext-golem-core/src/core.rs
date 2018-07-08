use std::sync::{mpsc, Arc};
use std::thread;

use actix::prelude::*;
use actix::Syn;
use cpython::*;
use futures::Future;
use spin;

use net::event::Event;
use net::codec::message::{Encapsulated, Message};
use net::transport::message::*;
use net::transport::tcp::TcpTransport;
use net::transport::udp::UdpTransport;
use net::transport::*;

use error::ModuleError;
use network::*;
use python::*;

const CHANNEL_SIZE: usize = 2048;

pub struct Core {
    pub network: Option<Addr<Syn, NetworkCore>>,
    // Makes the Receiver Sync; required to hand off execution to Python's VM
    pub rx: Option<Arc<spin::Mutex<mpsc::Receiver<Event>>>>,
}

impl Core {
    pub fn run(
        &mut self,
        py: Python,
        py_host: PyString,
        py_port: PyLong,
    ) -> Result<(), ModuleError> {
        // initialize and assign Python context
        let address = to_socket_address(py, py_host, py_port)?;

        // start callback channel
        let (tx_queue, rx_queue) = mpsc::sync_channel(CHANNEL_SIZE);
        // start initialization channel
        let (tx, rx) = mpsc::channel();

        // spawn the network thread
        thread::spawn(move || {
            let sys = System::new("net");
            let (unsync, syn) = NetworkCore::run(tx_queue);

            if let Err(e) = TcpTransport::run(unsync.clone(), address.clone()) {
                let e = ModuleError::from(e);
                tx.send(Err(e)).ok();
                return;
            }

            if let Err(e) = UdpTransport::run(unsync.clone(), address.clone()) {
                let e = ModuleError::from(e);
                tx.send(Err(e)).ok();
                return;
            }

            if let Err(_) = tx.send(Ok(syn)) {
                return;
            }

            sys.run();
        });

        // retrieve the (syn) network actor address
        let result = rx.recv()?;
        match result {
            Ok(a) => {
                self.network = Some(a);
                self.rx = Some(Arc::new(spin::Mutex::new(rx_queue)));
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub fn running(&self) -> bool {
        match self.network {
            Some(_) => true,
            None => false,
        }
    }

    pub fn stop(&self) -> Result<(), ModuleError> {
        self.forward(Stop(TransportProtocol::Tcp))?;
        self.forward(Stop(TransportProtocol::Udp))?;
        Ok(())
    }
}

impl Core {
    pub fn connect(
        &self,
        py: Python,
        py_protocol: PyLong,
        py_host: PyString,
        py_port: PyLong,
    ) -> Result<(), ModuleError> {
        let address = to_socket_address(py, py_host, py_port)?;
        let protocol: u16 = py_extract!(py, py_protocol)?;
        let transport = TransportProtocol::from(protocol);

        self.forward(Connect { transport, address })
    }

    pub fn disconnect(
        &self,
        py: Python,
        py_protocol: PyLong,
        py_host: PyString,
        py_port: PyLong,
    ) -> Result<(), ModuleError> {
        let address = to_socket_address(py, py_host, py_port)?;
        let protocol: u16 = py_extract!(py, py_protocol)?;
        let transport = TransportProtocol::from(protocol);

        self.forward(Disconnect { transport, address })
    }

    pub fn send(
        &self,
        py: Python,
        py_protocol: PyLong,
        py_host: PyString,
        py_port: PyLong,
        py_protocol_id: PyLong,
        py_message: PyBytes,
    ) -> Result<(), ModuleError> {
        let protocol: u16 = py_extract!(py, py_protocol)?;
        let protocol_id: u16 = py_extract!(py, py_protocol_id)?;
        let address = to_socket_address(py, py_host, py_port)?;
        let message = py_message.data(py);

        self.forward(SendMessage {
            transport: TransportProtocol::from(protocol),
            address,
            message: Message::Encapsulated(Encapsulated {
                protocol_id,
                message: message.to_vec(),
            }),
        })
    }
}

impl Core {
    fn forward<M>(&self, msg: M) -> Result<(), ModuleError>
    where
        M: actix::Message + Send + 'static,
        M::Result: Send,
        NetworkCore: actix::Handler<M>,
    {
        match self.network {
            Some(ref n) => {
                n.send(msg).wait()?;
                Ok(())
            }
            None => Err(ModuleError::from(MailboxError::Closed)),
        }
    }
}
