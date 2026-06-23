use bincode::{Decode, Encode, config::standard, encode_to_vec};
use std::{io, sync::Arc};
use tokio::{
    net::UdpSocket,
    runtime::Runtime,
    sync::mpsc::{Receiver, Sender, channel},
};

pub struct NetworkClient {
    _rt: Runtime,
    pub sender: Sender<Vec<u8>>,
    pub receiver: Receiver<Vec<u8>>,
}

impl NetworkClient {
    pub fn new(addr: &str, remote_addr: &str) -> io::Result<Self> {
        let rt = Runtime::new()?;

        let (tx_to_net, mut rx_to_net) = channel::<Vec<u8>>(100);
        let (tx_from_net, rx_from_net) = channel::<Vec<u8>>(100);

        let addr = addr.to_string();
        let remote_addr = remote_addr.to_string();

        rt.spawn(async move {
            let sock = UdpSocket::bind(addr).await.unwrap();
            let sock = Arc::new(sock);

            sock.connect(remote_addr).await.unwrap();

            let sock_send = Arc::clone(&sock);
            tokio::spawn(async move {
                while let Some(data) = rx_to_net.recv().await {
                    if let Err(e) = sock_send.send(&data).await {
                        eprintln!("Erro ao enviar UDP: {}", e);
                    }
                }
            });
            let sock_recv = Arc::clone(&sock);
            tokio::spawn(async move {
                let mut buf = [0; 1024];
                loop {
                    match sock_recv.recv(&mut buf).await {
                        Ok(len) => {
                            if tx_from_net.send(buf[..len].to_vec()).await.is_err() {
                                break;
                            }
                        }
                        Err(e) => {
                            eprintln!("Erro na leitura do UDP: {}", e);
                            break;
                        }
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
            let server_event = bincode::decode_from_slice(&event, config).unwrap();
            values.push(server_event.0);
        }
        values
    }
    pub fn send<T: Decode<()> + Encode + 'static>(&self, event: T) {
        let config = standard();
        let bytes = encode_to_vec(event, config).unwrap();
        let _ = self.sender.try_send(bytes);
    }
}
