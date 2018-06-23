extern crate actix;
extern crate bincode;
extern crate byteorder;
extern crate bytes;
extern crate futures;
extern crate rand;
extern crate serde;
extern crate tokio;
extern crate tokio_codec;
extern crate tokio_io;
extern crate tokio_tcp;
extern crate tokio_udp;

extern crate net;

use std::net::SocketAddr;
use std::time::{Duration, Instant};
use std::{env, process};

use actix::prelude::*;
use actix::Unsync;
use futures::{future, Future};
use rand::Rng;
use tokio::timer::{DeadlineError, Delay};

use net::codec::message::{Encapsulated, Message};
use net::network::session::*;
use net::network::*;
use net::transport::message::*;
use net::transport::tcp::*;
use net::transport::udp::*;
use net::transport::*;

//
// Misc.
//

fn rand_message() -> Message {
    let mut rng = rand::thread_rng();

    let protocol_id: u16 = rng.gen_range(0, 16535);
    let message: Vec<u8> = (0..32).map(|_| rng.gen_range(0, 255)).collect();

    Message::Encapsulated(Encapsulated {
        protocol_id,
        message,
    })
}

//
// Application logic
//

struct App {
    sessions: Sessions<App>,
    tcp: Option<TcpActorAddr<App>>,
    udp: Option<UdpActorAddr<App>>,
}

impl App {
    pub fn run() -> NetAddr<App> {
        App::create(|_| App {
            sessions: Sessions::new(),
            tcp: None,
            udp: None,
        })
    }

    fn transport_send<M, D>(&self, transport: &Option<Addr<Unsync, D>>, message: M) -> EmptyResult
    where
        M: actix::Message + 'static,
        D: Actor + Handler<M>,
        D::Context: AsyncContext<D>,
    {
        match transport {
            Some(ref t) => {
                let future = t
                    .send(message)
                    .map(|_| ())
                    .map_err(move |e| eprintln!("Error sending message (transport): {}", e));

                Arbiter::handle().spawn(future);
                Ok(())
            }
            None => Err(MailboxError::Closed),
        }
    }

    fn session_send<M, D>(session: &Addr<Unsync, D>, message: M)
    where
        M: actix::Message + 'static,
        D: Actor + Handler<M>,
        D::Context: AsyncContext<D>,
    {
        let future = session
            .send(message)
            .map(|_| ())
            .map_err(move |e| eprintln!("Error sending message (session): {}", e));

        Arbiter::handle().spawn(future);
    }

    fn send_random_message(&self, transport: &TransportProtocol, address: &SocketAddr) {
        println!("Send random message");

        let session = match self.sessions.get(transport, address) {
            Some(s) => s.clone(),
            None => {
                eprintln!("Unable to reply to a received message");
                return;
            }
        };

        let message = SessionSendMessage {
            address: address.clone(),
            message: rand_message(),
        };

        let deadline = Instant::now() + Duration::from_millis(750);
        let future = Delay::new(deadline)
            .then(move |_| {
                println!("Sending: {:?}", message);

                match session {
                    TransportSession::Tcp(s) => Self::session_send(&s, message),
                    TransportSession::Udp(s) => Self::session_send(&s, message),
                };

                future::ok(())
            })
            .map(|_| ())
            .map_err(move |e: DeadlineError<()>| {
                eprintln!("Message send delay error: {:?}", e);
            });

        Arbiter::handle().spawn(future);
    }
}

impl Network for App {}

impl Actor for App {
    type Context = Context<Self>;
}

// Forward
impl Handler<Stop> for App {
    type Result = EmptyResult;

    fn handle(&mut self, m: Stop, _ctx: &mut Self::Context) -> Self::Result {
        println!("App: stop");

        match m.0 {
            TransportProtocol::Tcp => self.transport_send(&self.tcp, m)?,
            TransportProtocol::Udp => self.transport_send(&self.udp, m)?,
            _ => return Err(MailboxError::Closed),
        };
        Ok(())
    }
}

// Event
impl Handler<Stopped<App>> for App {
    type Result = ();

    fn handle(&mut self, m: Stopped<App>, ctx: &mut Self::Context) {
        match m.0 {
            Transport::Tcp(_) => {
                println!("App: TCP transport stopped");
                self.tcp = None;
            }
            Transport::Udp(_) => {
                println!("App: UDP transport stopped");
                self.udp = None;
            }
        };

        if let None = self.tcp {
            if let None = self.udp {
                println!("App: shutting down");
                ctx.stop();
            }
        }
    }
}

// Event
impl Handler<ReceivedMessage> for App {
    type Result = ();

    fn handle(&mut self, m: ReceivedMessage, _ctx: &mut Self::Context) {
        println!("Received: {:?}", m);
        self.send_random_message(&m.transport, &m.address);
    }
}

// Forward
impl Handler<Connect> for App {
    type Result = EmptyResult;

    fn handle(&mut self, m: Connect, _ctx: &mut Self::Context) -> Self::Result {
        println!("App: connect {:?}", m);

        match m.transport {
            TransportProtocol::Tcp => {
                self.transport_send(&self.tcp, m)?;
                Ok(())
            }
            _ => return Err(MailboxError::Closed),
        }
    }
}

// Event
impl Handler<Connected<App>> for App {
    type Result = NoResult;

    fn handle(&mut self, m: Connected<App>, _ctx: &mut Self::Context) {
        println!("App: connected to {} ({})", m.address, m.transport);

        let transport = m.transport.clone();
        let address = m.address.clone();

        self.sessions.add(m.transport, m.address, m.session);

        if m.initiator {
            self.send_random_message(&transport, &address);
        }
    }
}

// Forward
impl Handler<Disconnect> for App {
    type Result = EmptyResult;

    fn handle(&mut self, m: Disconnect, _ctx: &mut Self::Context) -> Self::Result {
        println!("App: disconnect {} ({})", m.address, m.transport);

        let message = Stop(m.transport);

        match self.sessions.get(&m.transport, &m.address) {
            Some(session) => match session {
                TransportSession::Tcp(s) => {
                    Self::session_send(&s, message);
                }
                TransportSession::Udp(_) => {
                    return Err(MailboxError::Closed);
                }
            },
            None => {
                return Err(MailboxError::Closed);
            }
        };

        Ok(())
    }
}

// Event
impl Handler<Disconnected> for App {
    type Result = ();

    fn handle(&mut self, m: Disconnected, _ctx: &mut Self::Context) {
        println!("App: disconnected {} ({})", m.address, m.transport);
        self.sessions.remove(&m.transport, &m.address);
    }
}

// Event
impl Handler<Listening<App>> for App {
    type Result = ();

    fn handle(&mut self, m: Listening<App>, _ctx: &mut Self::Context) {
        match m.0 {
            Transport::Tcp(t) => {
                println!("App: TCP transport registered");
                self.tcp = Some(t);
            }
            Transport::Udp(t) => {
                println!("App: UDP transport registered");
                self.udp = Some(t);
            }
        };
    }
}

// Event
impl Handler<SendMessage> for App {
    type Result = EmptyResult;

    fn handle(&mut self, m: SendMessage, _ctx: &mut Self::Context) -> Self::Result {
        println!("App: send message");

        let message = SessionSendMessage {
            address: m.address,
            message: m.message,
        };

        if let TransportProtocol::Udp = m.transport {
            return self.transport_send(&self.udp, message);
        }

        match self.sessions.get(&m.transport, &m.address) {
            Some(ref s) => {
                match s {
                    TransportSession::Tcp(s) => Self::session_send(&s, message),
                    _ => return Err(MailboxError::Closed),
                };
            }
            None => return Err(MailboxError::Closed),
        };

        Ok(())
    }
}

//
// Helpers
//

fn run(address: SocketAddr, to_connect: Option<SocketAddr>) {
    println!("Starting on {}", address);

    let sys = System::new("testbed");
    let app = App::run();

    match TcpTransport::run(app.clone(), address.clone()) {
        Ok(actor) => {
            println!("TCP is listening");
            actor
        }
        Err(error) => {
            eprintln!("Error listening on {} (TCP): {}", address, error);
            return;
        }
    };

    match UdpTransport::run(app.clone(), address.clone()) {
        Ok(actor) => {
            println!("UDP is listening");
            actor
        }
        Err(error) => {
            eprintln!("Error listening on {} (UDP): {}", address, error);
            return;
        }
    };

    if let Some(a) = to_connect {
        println!("Connecting to {}", a);

        let message = Connect {
            transport: TransportProtocol::Tcp,
            address: a,
        };

        let deadline = Instant::now() + Duration::from_millis(1000);
        let future = Delay::new(deadline)
            .then(move |_| {
                let f = app
                    .send(message)
                    .map(|_| ())
                    .map_err(move |e| eprintln!("Error sending message: {}", e));

                Arbiter::handle().spawn(f);

                future::ok(())
            })
            .map(|_| ())
            .map_err(move |e: DeadlineError<()>| {
                eprintln!("Message send delay error: {:?}", e);
            });

        Arbiter::handle().spawn(future);
    }

    sys.run();
}

//
// Entry point
//

pub fn main() {
    fn usage(name: &str) {
        println!("Usage: {} host:port [host:port]", name);
        process::exit(1);
    }

    fn parse_address_arg(string: &String) -> Option<SocketAddr> {
        match string.parse() {
            Ok(address) => Some(address),
            Err(_) => None,
        }
    }

    let args: Vec<String> = env::args().collect();

    match args.len() {
        2 => match parse_address_arg(&args[1]) {
            Some(a) => {
                run(a, None);
            }
            None => {
                usage(&args[0]);
            }
        },
        3 => match parse_address_arg(&args[1]) {
            Some(a) => {
                run(a, parse_address_arg(&args[2]));
            }
            None => {
                usage(&args[0]);
            }
        },
        _ => {
            usage(&args[0]);
        }
    };
}
