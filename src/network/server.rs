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

use crate::{NetworkEvent, ServerEvent, serialize_bytes};

pub struct NetworkServer {
    pub sender: Sender<(SocketAddr, ServerEvent)>,
    pub receiver: Receiver<(SocketAddr, ServerEvent)>,
}

impl NetworkServer {
    pub fn new(addr: &str, handle: &Handle) -> io::Result<Self> {
        let (tx_to_net, mut rx_to_net) = channel::<(SocketAddr, ServerEvent)>(100);
        let (tx_from_net, rx_from_net) = channel::<(SocketAddr, ServerEvent)>(100);

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
                        let serialized_event = serialize_bytes(&data);
                        let _ = client_tx.send(serialized_event).await;
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

                        let encoded_connect = serialize_bytes(&NetworkEvent::Connected);
                        let _ = tx_from_net_clone
                            .send((peer_addr, ServerEvent::Broadcast(encoded_connect)))
                            .await;

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
                                    let server_event = ServerEvent::Broadcast(data.to_vec());
                                    if tx_from_net_clone
                                        .send((peer_addr, server_event))
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

                        let encoded_disconnect = serialize_bytes(&NetworkEvent::Disconnected);

                        let _ = tx_from_net_clone
                            .send((peer_addr, ServerEvent::Broadcast(encoded_disconnect)))
                            .await;
                    }
                });
            }
        });

        Ok(Self {
            sender: tx_to_net,
            receiver: rx_from_net,
        })
    }

    pub fn drain(&mut self) -> Vec<(SocketAddr, ServerEvent)> {
        let mut values = Vec::new();
        while let Ok((socket, event)) = self.receiver.try_recv() {
            values.push((socket, event));
        }
        values
    }
    pub fn send(&self, target: SocketAddr, event: ServerEvent) {
        let _ = self.sender.try_send((target, event));
    }
}
