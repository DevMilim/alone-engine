use bincode::{Decode, Encode, config::standard, encode_to_vec};
use futures_util::{SinkExt, StreamExt};
use std::io;
use tokio::{
    runtime::Handle,
    sync::mpsc::{Receiver, Sender, channel},
};
use tokio_tungstenite::{connect_async, tungstenite::Message};

pub struct NetworkClient {
    pub sender: Sender<Vec<u8>>,
    pub receiver: Receiver<Vec<u8>>,
}

impl NetworkClient {
    pub fn new(server_url: &str, handle: &Handle) -> io::Result<Self> {
        let (tx_to_net, mut rx_to_net) = channel::<Vec<u8>>(100);
        let (tx_from_net, rx_from_net) = channel::<Vec<u8>>(100);

        let url = if !server_url.starts_with("ws://") && !server_url.starts_with("wss://") {
            format!("ws://{}", server_url)
        } else {
            server_url.to_string()
        };

        handle.spawn(async move {
            match connect_async(&url).await {
                Ok((ws_stream, _response)) => {
                    println!("Conectado ao servidor: {}", url);

                    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
                    let tx_task = tokio::spawn(async move {
                        while let Some(data) = rx_to_net.recv().await {
                            if ws_sender.send(Message::Binary(data.into())).await.is_err() {
                                eprintln!("Conexão perdida");
                                break;
                            }
                        }
                    });
                    while let Some(msg) = ws_receiver.next().await {
                        match msg {
                            Ok(Message::Binary(data)) => {
                                if tx_from_net.send(data.into()).await.is_err() {
                                    break;
                                }
                            }
                            Ok(Message::Close(_)) | Err(_) => {
                                println!("desconectado do servidor");
                                break;
                            }
                            _ => {}
                        }
                    }
                    tx_task.abort();
                }
                Err(_) => eprintln!("Falha ao conectar ao servidor"),
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
            if let Ok((server_event, _)) = bincode::decode_from_slice::<T, _>(&event, config) {
                values.push(server_event);
            }
        }
        values
    }
    pub fn send<T: Decode<()> + Encode + 'static>(&self, event: T) {
        let config = standard();
        let bytes = encode_to_vec(event, config).unwrap();
        let _ = self.sender.try_send(bytes);
    }
}
