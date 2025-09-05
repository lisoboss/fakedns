use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::{
    net::UdpSocket,
    select,
    sync::{mpsc, oneshot, Mutex},
};

use crate::{cancel, handle, payload::Payload};

type Response = oneshot::Sender<Payload>;

#[derive(Debug)]
pub enum DnsCommand {
    Query { payload: Payload, resp: Response },
    TimedOut { id: u16 },
}

#[derive(Debug, Clone)]
pub struct Dns {
    sock: Arc<UdpSocket>,
    map: Arc<Mutex<HashMap<u16, Response>>>,
}

impl Dns {
    pub async fn new(remote_addr: &str) -> Self {
        let sock = UdpSocket::bind("0.0.0.0:0")
            .await
            .expect("[E] dns bind 0.0.0.0:0");
        #[cfg(debug_assertions)]
        println!("[+] dns bind 0:{}", sock.local_addr().unwrap().port());

        let remote_addr = remote_addr
            .parse::<SocketAddr>()
            .expect("[E] dns remote_addr parse");
        sock.connect(remote_addr)
            .await
            .expect("[E] dns connect remote_addr");
        #[cfg(debug_assertions)]
        println!("[+] dns connect {remote_addr:?}");

        Self {
            sock: Arc::new(sock),
            map: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn work_cmd(&self, mut rx: mpsc::Receiver<DnsCommand>) {
        #[cfg(debug_assertions)]
        println!("[+] dns work cmd");

        while let Some(cmd) = rx.recv().await {
            match cmd {
                DnsCommand::TimedOut { id } => {
                    let mut map = self.map.lock().await;
                    let _ = map.remove(&id);
                }
                DnsCommand::Query { payload, resp } => {
                    let mut map = self.map.lock().await;
                    handle!(handle!(cancel!(self.sock.send(&payload.as_ref()), 3), e => {
                        println!("[E] dns request send {e:?}");
                        continue;
                    }), e => {
                        println!("[E] dns request send {e:?}");
                        continue;
                    });
                    let _ = map.insert(payload.id(), resp);
                }
            }
        }
    }

    pub async fn work_response(&self) {
        #[cfg(debug_assertions)]
        println!("[+] dns work response");

        let mut buf = [0; 1024];
        while let Ok(len) = self.sock.recv(&mut buf).await {
            let payload = Payload::from(&buf[..len]);
            let mut map = self.map.lock().await;
            if let Some(response) = map.remove(&payload.id()) {
                if let Err(e) = response.send(payload) {
                    println!("[E] raw response send {e:?}");
                }
            }
        }
    }
}
