use std::{
    error::Error,
    iter::{once, Once},
    net::{SocketAddr, TcpListener, ToSocketAddrs, UdpSocket},
    thread::{spawn, JoinHandle},
};

// ServerInterface should always be running on the same machine as steamvr so i will use udp for speed as it is unlikely to fail, as no wifi is involved
pub struct ServerInterface {
    socket: UdpSocket,
    maingate: TcpListener,
    clients: Vec<Client>,
    rename_me: Option<JoinHandle<()>>,
}

impl ServerInterface {
    pub fn new(socket_addr: SocketAddr) -> Result<Self, Box<dyn Error>> {
        let socket = UdpSocket::bind(socket_addr)?;
        let maingate = TcpListener::bind(socket_addr)?;
        Ok(Self {
            socket,
            maingate,
            clients: Vec::new(),
            rename_me: None,
        })
    }

    pub fn start(&mut self) {
        self.rename_me = Some(spawn(|| Self::main()))
    }

    fn main() {
        // do tcp thingy do dah

        // client should send id to listener
        // assumming the interface is not running to many clients to prevent a dos-ing, accept the client
    }

    pub fn stop(&mut self) {
        todo!()
    }
}

/// Represnts a client connected to the server
pub struct Client {
    socket: SocketAddr,
    id: u64,
}

impl ToSocketAddrs for Client {
    type Iter = Once<SocketAddr>;

    fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
        Ok(once(self.socket))
    }
}
