use bincode::config::standard;
use bincode::{Decode, Encode, encode_to_vec};
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::{io, sync::Arc};
use tokio::net::TcpListener;
use tokio::runtime::Handle;
use tokio::sync::Mutex;
use tokio::sync::mpsc::{Receiver, Sender, channel};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

pub struct NetworkServer {
    pub sender: Sender<(SocketAddr, Vec<u8>)>,
    pub receiver: Receiver<(SocketAddr, Vec<u8>)>,
}

impl NetworkServer {
    pub fn new(addr: &str, handle: &Handle) -> io::Result<Self> {
        let (tx_to_net, mut rx_to_net) = channel::<(SocketAddr, Vec<u8>)>(100);
        let (tx_from_net, rx_from_net) = channel::<(SocketAddr, Vec<u8>)>(100);

        let addr = addr.to_string();

        handle.spawn(async move {
            let listener = TcpListener::bind(addr).await.unwrap();
            let active_clients: Arc<Mutex<HashMap<SocketAddr, Sender<Vec<u8>>>>> =
                Arc::new(Mutex::new(HashMap::new()));

            // Envio de dados
            let clients_for_send = Arc::clone(&active_clients);
            tokio::spawn(async move {
                while let Some((target_addr, data)) = rx_to_net.recv().await {
                    let clients_guard = clients_for_send.lock().await;
                    if let Some(client_tx) = clients_guard.get(&target_addr) {
                        let _ = client_tx.send(data).await;
                    } else {
                        eprintln!("Cliente não conectado")
                    }
                }
            });

            while let Ok((stream, peer_addr)) = listener.accept().await {
                let tx_from_net_clone = tx_from_net.clone();
                let clients_clone = Arc::clone(&active_clients);

                tokio::spawn(async move {
                    if let Ok(ws_stream) = accept_async(stream).await {
                        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

                        let (client_tx, mut client_rx) = channel::<Vec<u8>>(100);

                        clients_clone.lock().await.insert(peer_addr, client_tx);

                        let send_task = tokio::spawn(async move {
                            while let Some(data) = client_rx.recv().await {
                                if ws_sender.send(Message::Binary(data.into())).await.is_err() {
                                    break;
                                }
                            }
                        });
                        while let Some(msg) = ws_receiver.next().await {
                            match msg {
                                Ok(Message::Binary(data)) => {
                                    if tx_from_net_clone
                                        .send((peer_addr, data.to_vec()))
                                        .await
                                        .is_err()
                                    {
                                        break;
                                    }
                                }
                                Ok(Message::Close(_)) | Err(_) => {
                                    break;
                                }
                                _ => {}
                            }
                        }
                        send_task.abort();
                        clients_clone.lock().await.remove(&peer_addr);
                    }
                });
            }
        });

        Ok(Self {
            sender: tx_to_net,
            receiver: rx_from_net,
        })
    }

    pub fn drain<T: Decode<()> + Encode + 'static>(&mut self) -> Vec<T> {
        let config = standard();
        let mut values = Vec::new();
        while let Ok(event) = self.receiver.try_recv() {
            if let Ok((server_event, _)) = bincode::decode_from_slice::<T, _>(&event.1, config) {
                values.push(server_event);
            }
        }
        values
    }
    pub fn send<T: Decode<()> + Encode + 'static>(&self, target: SocketAddr, event: T) {
        let config = standard();
        let bytes = encode_to_vec(event, config).unwrap();
        let _ = self.sender.try_send((target, bytes));
    }
}
