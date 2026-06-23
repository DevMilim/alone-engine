use bincode::config::standard;
use bincode::{Decode, Encode, encode_to_vec};
use std::net::SocketAddr;
use std::{io, sync::Arc};
use tokio::{
    net::UdpSocket,
    runtime::Runtime,
    sync::mpsc::{Receiver, Sender, channel},
};

pub struct NetworkServer {
    _rt: Runtime,
    pub sender: Sender<(SocketAddr, Vec<u8>)>,
    pub receiver: Receiver<(SocketAddr, Vec<u8>)>,
}

impl NetworkServer {
    pub fn new(addr: &str) -> io::Result<Self> {
        let rt = Runtime::new()?;

        let (tx_to_net, mut rx_to_net) = channel::<(SocketAddr, Vec<u8>)>(100);
        let (tx_from_net, rx_from_net) = channel::<(SocketAddr, Vec<u8>)>(100);

        let addr = addr.to_string();

        rt.spawn(async move {
            let sock = UdpSocket::bind(addr).await.unwrap();
            let sock = Arc::new(sock);

            // Envio de dados
            let sock_send = Arc::clone(&sock);
            tokio::spawn(async move {
                while let Some((target_addr, data)) = rx_to_net.recv().await {
                    if let Err(e) = sock_send.send_to(&data, target_addr).await {
                        eprintln!("Erro ao enviar UDP para {}: {}", target_addr, e);
                    }
                }
            });

            // recebimento de dados
            let sock_recv = Arc::clone(&sock);
            tokio::spawn(async move {
                let mut buf = [0; 1024];
                loop {
                    match sock_recv.recv_from(&mut buf).await {
                        Ok((len, peer_addr)) => {
                            if tx_from_net
                                .send((peer_addr, buf[..len].to_vec()))
                                .await
                                .is_err()
                            {
                                break;
                            }
                        }
                        Err(_) => {}
                    }
                }
            });
        });

        Ok(Self {
            _rt: rt,
            sender: tx_to_net,
            receiver: rx_from_net,
        })
    }

    pub fn drain<T: Decode<()> + Encode + 'static>(&mut self) -> Vec<T> {
        let config = standard();
        let mut values = Vec::new();
        while let Ok(event) = self.receiver.try_recv() {
            let server_event = bincode::decode_from_slice(&event.1, config).unwrap();
            values.push(server_event.0);
        }
        values
    }
    pub fn send<T: Decode<()> + Encode + 'static>(&self, target: SocketAddr, event: T) {
        let config = standard();
        let bytes = encode_to_vec(event, config).unwrap();
        let _ = self.sender.try_send((target, bytes));
    }
}
