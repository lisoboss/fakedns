use std::{
    collections::HashMap,
    io,
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{
    net::UdpSocket,
    sync::{mpsc, oneshot, Mutex, RwLock},
};

use crate::{cancel, handle, payload::Payload};

type Response = oneshot::Sender<Payload>;

const CACHE_TTL: Duration = Duration::from_secs(60);
const CACHE_MAX_SIZE: usize = 4096;

#[derive(Debug)]
struct CacheEntry {
    payload: Payload,
    expires_at: Instant,
}

fn domain_key(payload: &Payload) -> (u16, Vec<u8>) {
    let (_, offset) = payload.domain();
    let qtype = if payload.0.len() > offset + 2 {
        (payload.0[offset + 1] as u16) << 8 | payload.0[offset + 2] as u16
    } else {
        0
    };
    let domain_bytes = payload.0[12..=offset].to_vec();
    (qtype, domain_bytes)
}

#[derive(Debug)]
pub enum DnsCommand {
    Query { payload: Payload, resp: Response },
    TimedOut { id: u16 },
}

#[derive(Debug, Clone)]
pub struct Dns {
    sock: Arc<RwLock<UdpSocket>>,
    map: Arc<Mutex<HashMap<u16, (Response, (u16, Vec<u8>))>>>,
    cache: Arc<Mutex<HashMap<(u16, Vec<u8>), CacheEntry>>>,
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
            sock: Arc::new(RwLock::new(sock)),
            map: Arc::new(Mutex::new(HashMap::new())),
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn recover_sock(&self, err: io::Error) {
        if err.kind() == io::ErrorKind::AddrNotAvailable {
            let mut sock = self.sock.write().await;
            *sock = UdpSocket::bind("0.0.0.0:0").await.unwrap();
            println!("[+] dns reset due to AddrNotAvailable");
        }
    }

    async fn hit_cache(&self, key: (u16, Vec<u8>), payload_id: u16) -> Option<Payload> {
        let mut cache = self.cache.lock().await;
        if let Some(entry) = cache.get_mut(&key) {
            if entry.expires_at > Instant::now() {
                entry.expires_at = Instant::now() + CACHE_TTL;
                let mut cached = entry.payload.clone();
                cached.0[0] = (payload_id >> 8) as u8;
                cached.0[1] = (payload_id & 0xFF) as u8;
                return Some(cached);
            } else {
                cache.remove(&key);
            }
        }
        None
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
                DnsCommand::Query { mut payload, resp } => {
                    let key = domain_key(&payload);
                    if let Some(cached) = self.hit_cache(key.clone(), payload.id()).await {
                        #[cfg(debug_assertions)]
                        println!(
                            "[+] dns cache hit for id {} {}",
                            payload.id(),
                            payload
                                .domain()
                                .0
                                .iter()
                                .map(|l| String::from_utf8_lossy(l))
                                .collect::<Vec<_>>()
                                .join(".")
                        );

                        if let Err(e) = resp.send(cached) {
                            println!("[E] raw response send {e:?}");
                        }
                        continue;
                    }

                    let mut map = self.map.lock().await;
                    let sock = self.sock.read().await;
                    handle!(handle!(cancel!(sock.send(&payload.as_ref()), 3), e => {
                        println!("[E] dns request send cancel {e:?}");
                        payload.servfail();
                        if let Err(e) = resp.send(payload) {
                            println!("[E] raw response send {e:?}");
                        };
                        continue;
                    }), e => {
                        println!("[E] dns request send {e:?}");
                        payload.servfail();
                        if let Err(e) = resp.send(payload) {
                            println!("[E] raw response send {e:?}");
                        };
                        self.recover_sock(e).await;
                        continue;
                    });

                    let _ = map.insert(payload.id(), (resp, key));
                }
            }
        }
    }

    pub async fn work_response(&self) {
        #[cfg(debug_assertions)]
        println!("[+] dns work response");

        let mut buf = vec![0; 1024];
        loop {
            let len = match self.sock.read().await.recv(&mut buf).await {
                Ok(len) => len,
                Err(e) => {
                    println!("[E] dns response recv {e:?}");
                    continue;
                }
            };

            let payload = Payload::from(&buf[..len]);

            let sender = {
                let mut map = self.map.lock().await;
                match map.remove(&payload.id()) {
                    Some((sender, key)) => {
                        let mut cache = self.cache.lock().await;
                        if cache.len() >= CACHE_MAX_SIZE {
                            if let Some(oldest_key) = cache
                                .iter()
                                .min_by_key(|(_, e)| e.expires_at)
                                .map(|(k, _)| k.clone())
                            {
                                cache.remove(&oldest_key);
                            }
                        }
                        cache.insert(
                            key,
                            CacheEntry {
                                payload: payload.clone(),
                                expires_at: Instant::now() + CACHE_TTL,
                            },
                        );
                        sender
                    }
                    None => {
                        println!("[E] raw response id {} not found", payload.id());
                        continue;
                    }
                }
            };

            if let Err(e) = sender.send(payload) {
                println!("[E] raw response send {e:?}");
            }
        }
    }
}
