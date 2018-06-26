use actix;
use actix::msgs;
use actix::prelude::*;
use actix::{Addr, Unsync};
use futures::sync::mpsc::UnboundedSender;
use futures::Future;

use net::codec::message::Message;
use net::network::session::*;
use net::network::*;
use net::transport::message::*;
use net::transport::tcp::TcpActorAddr;
use net::transport::udp::UdpActorAddr;
use net::transport::*;

use event::CoreEvent;

pub struct NetworkCore {
    sessions: Sessions<NetworkCore>,
    tcp: Option<TcpActorAddr<NetworkCore>>,
    udp: Option<UdpActorAddr<NetworkCore>>,
    tx: UnboundedSender<CoreEvent>,
}

impl NetworkCore {
    pub fn run(tx: UnboundedSender<CoreEvent>) -> (Addr<Unsync, Self>, Addr<Syn, Self>) {
        NetworkCore::create(|_| NetworkCore {
            sessions: Sessions::new(),
            tcp: None,
            udp: None,
            tx,
        })
    }

    fn tx_send(&self, event: CoreEvent) {
        if let Err(e) = self.tx.unbounded_send(event) {
            eprintln!("Core: error sending event: {}", e);
        }
    }

    fn transport_send<M, D>(&self, transport: &Option<Addr<Unsync, D>>, message: M) -> EmptyResult
    where
        M: actix::Message + 'static,
        D: Actor + Handler<M>,
        D::Context: AsyncContext<D>,
    {
        match transport {
            Some(ref t) => {
                let future = t.send(message).map(|_| ()).map_err(move |_| {});

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
            .map_err(move |e| eprintln!("Core: error sending message: {}", e));

        Arbiter::handle().spawn(future);
    }
}

impl Network for NetworkCore {}

impl Actor for NetworkCore {
    type Context = Context<Self>;

    fn stopped(&mut self, _: &mut <Self as Actor>::Context) {
        Arbiter::system().do_send(msgs::SystemExit(0));
    }
}

// Forward
impl Handler<Stop> for NetworkCore {
    type Result = EmptyResult;

    fn handle(&mut self, m: Stop, ctx: &mut Self::Context) -> Self::Result {
        self.transport_send(&self.tcp, m.clone())?;
        self.transport_send(&self.udp, m)?;

        ctx.stop();
        Ok(())
    }
}

// Event
impl Handler<Stopped<NetworkCore>> for NetworkCore {
    type Result = NoResult;

    fn handle(&mut self, m: Stopped<NetworkCore>, ctx: &mut Self::Context) {
        match m.0 {
            Transport::Tcp(_) => {
                self.tx_send(CoreEvent::Stopped(TransportProtocol::Tcp));
                self.tcp = None;
            }
            Transport::Udp(_) => {
                self.tx_send(CoreEvent::Stopped(TransportProtocol::Udp));
                self.udp = None;
            }
        };

        if let None = self.tcp {
            if let None = self.udp {
                self.tx_send(CoreEvent::Exiting);
                ctx.stop();
            }
        }
    }
}

// Event
impl Handler<ReceivedMessage> for NetworkCore {
    type Result = NoResult;

    fn handle(&mut self, m: ReceivedMessage, _ctx: &mut Self::Context) {
        match m.message {
            Message::Encapsulated(e) => {
                let event = CoreEvent::Message(m.transport, m.address, e);
                self.tx_send(event);
            }
            _ => {}
        };
    }
}

// Forward
impl Handler<Connect> for NetworkCore {
    type Result = EmptyResult;

    fn handle(&mut self, m: Connect, _ctx: &mut Self::Context) -> Self::Result {
        match m.transport {
            TransportProtocol::Tcp => {
                self.transport_send(&self.tcp, m)?;
                Ok(())
            }
            _ => Err(MailboxError::Closed),
        }
    }
}

// Event
impl Handler<Connected<NetworkCore>> for NetworkCore {
    type Result = NoResult;

    fn handle(&mut self, m: Connected<NetworkCore>, _ctx: &mut Self::Context) {
        let event = CoreEvent::Connected(m.transport.clone(), m.address.clone());
        self.sessions.add(m.transport, m.address, m.session);
        self.tx_send(event);
    }
}

// Forward
impl Handler<Disconnect> for NetworkCore {
    type Result = EmptyResult;

    fn handle(&mut self, m: Disconnect, _ctx: &mut Self::Context) -> Self::Result {
        let message = Stop(m.transport);
        match self.sessions.get(&m.transport, &m.address) {
            Some(session) => match session {
                TransportSession::Tcp(s) => {
                    Self::session_send(&s, message);
                    Ok(())
                }
                TransportSession::Udp(_) => Err(MailboxError::Closed),
            },
            None => Err(MailboxError::Closed),
        }
    }
}

// Event
impl Handler<Disconnected> for NetworkCore {
    type Result = NoResult;

    fn handle(&mut self, m: Disconnected, _ctx: &mut Self::Context) {
        let event = CoreEvent::Disconnected(m.transport.clone(), m.address.clone());
        self.sessions.remove(&m.transport, &m.address);
        self.tx_send(event);
    }
}

// Event
impl Handler<Listening<NetworkCore>> for NetworkCore {
    type Result = NoResult;

    fn handle(&mut self, m: Listening<NetworkCore>, _ctx: &mut Self::Context) {
        let transport = match m.0 {
            Transport::Tcp(t) => {
                self.tcp = Some(t);
                TransportProtocol::Tcp
            }
            Transport::Udp(t) => {
                self.udp = Some(t);
                TransportProtocol::Udp
            }
        };

        let event = CoreEvent::Started(transport);
        self.tx_send(event);
    }
}

// Event
impl Handler<SendMessage> for NetworkCore {
    type Result = EmptyResult;

    fn handle(&mut self, m: SendMessage, _ctx: &mut Self::Context) -> Self::Result {
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
