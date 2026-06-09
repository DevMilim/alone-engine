use std::{
    thread::{JoinHandle, spawn},
    time::Duration,
};

use crossbeam_channel::{Receiver, Sender};

pub struct Network<RX: Send + 'static, TX: Send> {
    pub command_tx: Sender<RX>,
    pub event_rx: Receiver<TX>,
    pub stop_tx: Sender<()>,
    pub handle: Option<JoinHandle<()>>,
}
impl<RX: Send, TX: Send> Network<RX, TX> {
    pub fn new() -> Self {
        let (command_tx, command_rx) = crossbeam_channel::unbounded();
        let (event_tx, event_rx) = crossbeam_channel::unbounded();
        let (stop_tx, stop_rx) = crossbeam_channel::unbounded();

        let handle = spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap();

            rt.block_on(async move {
                loop {
                    if stop_rx.try_recv().is_ok() {
                        break;
                    }
                    while let Ok(cmd) = command_rx.try_recv() {
                        //let _ = event_tx.send(msg)
                    }

                    tokio::time::sleep(Duration::from_millis(1)).await;
                }
            });
        });

        Self {
            command_tx,
            event_rx,
            stop_tx,
            handle: Some(handle),
        }
    }
}
