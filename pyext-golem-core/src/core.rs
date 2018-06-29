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

struct Bridge {
    queue: Option<PyShared>,
    rx: Option<UnboundedReceiver<CoreEvent>>,
}

impl Bridge {
    pub fn run(queue: PyShared, rx: UnboundedReceiver<CoreEvent>) -> Addr<Unsync, Self> {
        Bridge::create(move |_ctx| Bridge {
            queue: Some(queue),
            rx: Some(rx),
        })
    }
}

impl Actor for Bridge {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut <Self as Actor>::Context) {
        let queue = self.queue.take().unwrap();
        let rx = self.rx.take().unwrap();

        let reader =
            rx.for_each(move |event| {
                let gil = Python::acquire_gil();
                let py = gil.python();

                let event = event.make_py_tuple(py);
                let args: PyTuple = py_wrap!(py, (event,));
                py_call_method!(py, queue, "put", args, None);

                Ok(())
            }).map_err(|_| eprintln!("Core event receiver error"));

        Arbiter::handle().spawn(reader);
    }
}

pub struct Core {
    pub network: Option<Addr<Syn, NetworkCore>>,
}

impl Core {
    pub fn run(
        &mut self,
        queue: PyShared,
        py_host: PyString,
        py_port: PyLong,
    ) -> Result<(), ModuleError> {
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

            Bridge::run(queue, rx);

            if let Err(_) = tx_init.send(Ok(syn)) {
                return;
            }

            sys.run();
        });

        // retrieve the (syn) network actor address
        let result = rx_init.recv()?;
        match result {
            Ok(a) => {
                self.network = Some(a);
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
}

impl Core {
    pub fn connect(
        &self,
        py_protocol: PyLong,
        py_host: PyString,
        py_port: PyLong,
    ) -> Result<(), ModuleError> {
        let address = to_socket_address(py_host, py_port)?;
        let protocol: u16 = py_extract!(py_protocol)?;
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
        let protocol: u16 = py_extract!(py_protocol)?;
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
        py_message: PyBytes,
    ) -> Result<(), ModuleError> {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let address = to_socket_address(py_host, py_port)?;
        let protocol: u16 = py_extract!(py, py_protocol)?;
        let protocol_id: u16 = py_extract!(py, py_protocol_id)?;
        let message = py_message.data(py);
        let message = message.to_vec();

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
                n.send(msg).wait()?;
                Ok(())
            }
            None => Err(MailboxError::Closed),
        }
    }
}
