use bincode::{Decode, Encode};
use macros::GameObject;

use crate::core::{Base, GameObject, GameObjectBase, Id};

#[derive(Debug, Encode, Decode, Clone)]
pub enum ServerEvent {
    Broadcast(Vec<u8>),
    Targeted(Id, Vec<u8>),
    Send(Id, Vec<u8>),
}

use futures_util::{SinkExt, StreamExt};
use std::io;
use tokio::{
    runtime::Handle,
    sync::mpsc::{Receiver, Sender, channel},
};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::{
    deserialize_bytes,
    objects::network::{NetworkError, NetworkEvent},
    serialize_bytes,
};

#[derive(GameObject)]
pub struct NetworkClient {
    #[base]
    base: Base,
    pub sender: Sender<ServerEvent>,
    pub receiver: Receiver<ServerEvent>,
}

impl NetworkClient {
    pub fn new(server_url: &str, handle: &Handle) -> io::Result<Self> {
        let (tx_to_net, mut rx_to_net) = channel::<ServerEvent>(100);
        let (tx_from_net, rx_from_net) = channel::<ServerEvent>(100);

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

                    let encoded_connected = serialize_bytes(&NetworkEvent::Connected);

                    let _ = tx_from_net
                        .send(ServerEvent::Broadcast(encoded_connected))
                        .await;

                    let tx_task = tokio::spawn(async move {
                        while let Some(data) = rx_to_net.recv().await {
                            let serialized_event = serialize_bytes(&data);
                            if ws_sender
                                .send(Message::Binary(serialized_event.into()))
                                .await
                                .is_err()
                            {
                                eprintln!("Conexão perdida");
                                break;
                            }
                        }
                    });
                    while let Some(msg) = ws_receiver.next().await {
                        match msg {
                            Ok(Message::Binary(data)) => {
                                if let Some(deserialized_event) =
                                    deserialize_bytes::<ServerEvent>(&data)
                                {
                                    if tx_from_net.send(deserialized_event).await.is_err() {
                                        break;
                                    }
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
                    let encoded_disconnected = serialize_bytes(&NetworkEvent::Disconnected);

                    let _ = tx_from_net
                        .send(ServerEvent::Broadcast(encoded_disconnected))
                        .await;
                }
                Err(_) => {
                    eprintln!("Falha ao conectar ao servidor");
                    let encoded = serialize_bytes(&NetworkEvent::ConnectFailed); // nova variante
                    let _ = tx_from_net.send(ServerEvent::Broadcast(encoded)).await;
                }
            }
        });
        Ok(Self {
            base: Base::default(),
            sender: tx_to_net,
            receiver: rx_from_net,
        })
    }
    pub fn drain(&mut self) -> Vec<ServerEvent> {
        let mut values = Vec::new();
        while let Ok(event) = self.receiver.try_recv() {
            values.push(event);
        }
        values
    }
    pub fn send(&self, event: ServerEvent) -> Result<(), NetworkError> {
        self.sender.try_send(event).map_err(|e| match e {
            tokio::sync::mpsc::error::TrySendError::Full(_) => NetworkError::ChannelFull,
            tokio::sync::mpsc::error::TrySendError::Closed(_) => NetworkError::Disconnected,
        })
    }
}

impl GameObject for NetworkClient {
    type Message = ();
}
