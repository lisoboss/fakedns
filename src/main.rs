mod dns;
mod macros;
mod payload;
mod trie;

use clap::Parser;
use std::{fs::read_to_string, net::SocketAddr, sync::Arc};
use tokio::{
    net::UdpSocket,
    select, spawn,
    sync::{mpsc, oneshot},
};

use crate::{
    dns::{Dns, DnsCommand},
    payload::Payload,
    trie::Trie,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Domain
    #[arg(short, long, default_value = "deploy/conf.d/domain.conf")]
    domain: String,

    /// ExcludeDomain
    #[arg(short, long, default_value = "deploy/conf.d/domain_exclude.conf")]
    exclude_domain: String,
}

const RESPONSE_START: &[u8] = &[0x81, 0x80, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01];
const RESPONSE_END: &[u8] = &[
    0xc0, 0x0c, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x01, 0xf4, 0x00, 0x04, 0x01, 0x02, 0x03, 0x04,
    0x00, 0x00, 0x29, 0x05, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
const MAX_BUFFER: usize = 5;

impl Trie {
    fn init_doamin(&mut self, domain_path: &str) {
        for line in read_to_string(domain_path).expect("[E] read file").lines() {
            let mut d: Vec<_> = line.trim().split(".").collect();
            d.reverse();
            self.insert(&d);
        }
    }

    pub fn is_domain(&self, domain: &[&str]) -> bool {
        self.starts_with(domain)
    }
}

fn fake_response(buf: &[u8], end_offset: usize) -> Vec<u8> {
    // "8fd6 0120 0001000000000001 0378723105766c70657203746f700000010001 0000291000000000000000"
    // "e116 0120 0001000000000001 02787205766c70657203746f700000010001 0000291000000000000000"

    // "e116 8180 0001000100000001 02787205766c70657203746f7000 00010001 c00c0001000100000001 0004 cebeee93 0000290580000000000000"
    // "9e31 8180 0001000100000001 02787205766c70657203746f7000 00010001 c00c00010001000001f4 0004 7f000001 0000290580000000000000"

    // "d939 0120 0001000000000001 0377777706676f6f676c6503636f6d00 00010001 0000291000000000000000"
    // "d939 8180 0001000100000001 0377777706676f6f676c6503636f6d00 000100   c00c00010001000001f400047f0000010000290580000000000000"
    // "61f5 8180 0001000100000001 0377777706676f6f676c6503636f6d00 00010001 c00c00010001000001f400047f0000010000290580000000000000"

    let mut r: Vec<u8> = Vec::new();
    r.extend_from_slice(&buf[..2]);
    r.extend_from_slice(RESPONSE_START);
    r.extend_from_slice(&buf[12..end_offset + 5]);
    r.extend_from_slice(RESPONSE_END);

    r
}

#[tokio::main]
async fn main() {
    let Args {
        domain,
        exclude_domain,
    } = Args::parse();
    println!("[+] domain: {domain:?}");
    println!("[+] exclude_domain: {exclude_domain:?}");

    let mut trie = Trie::new();
    let mut exclude_trie = Trie::new();
    trie.init_doamin(&domain);
    exclude_trie.init_doamin(&exclude_domain);

    let sock_local = Arc::new(UdpSocket::bind("0.0.0.0:53").await.expect("[E] bind 0:53"));
    println!("[+] bind: 53");

    let (tx_dns, rx_dns) = mpsc::channel::<DnsCommand>(MAX_BUFFER);
    let dns = Dns::new().await;

    let dns_c = dns.clone();
    spawn(async move { dns_c.work_cmd(rx_dns).await });
    spawn(async move { dns.work_response().await });

    let sock_local_c = sock_local.clone();
    let (tx_req, mut rx_req) = mpsc::channel::<(Payload, SocketAddr)>(MAX_BUFFER);
    spawn(async move {
        while let Some((payload, addr)) = rx_req.recv().await {
            #[cfg(debug_assertions)]
            println!("[+] {addr:?} send raw request");

            let (resp, rx) = oneshot::channel::<Payload>();
            let id = payload.id();
            tx_dns
                .send(DnsCommand::Query { payload, resp })
                .await
                .expect("[E] raw request dns cmd query");

            let payload = handle!(handle!(cancel!(rx, 5), e => {
                println!("[E] raw request dns rx {e:?} {addr:?}");
                tx_dns
                .send(DnsCommand::TimedOut { id })
                .await
                .expect("[E] raw request dns cmd timedout");
                continue;
            }), e => {
                println!("[E] raw request dns rx {e:?} {addr:?}");
                continue;
            });

            #[cfg(debug_assertions)]
            println!("[+] {addr:?} raw request id {} {}", id, payload.id());

            let _len = sock_local_c
                .send_to(payload.as_ref(), &addr)
                .await
                .expect("[E] raw response send_to");

            #[cfg(debug_assertions)]
            println!("[+] {addr:?} send raw response({_len:?})");
        }
    });

    let sock_local_c = sock_local.clone();
    let (tx, mut rx) = mpsc::channel::<(Payload, SocketAddr)>(MAX_BUFFER);
    spawn(async move {
        while let Some((payload, addr)) = rx.recv().await {
            let (domain, end_offset) = payload.domain();

            #[cfg(debug_assertions)]
            println!(
                "[+] {addr:?} domain {} offset {end_offset:?}",
                &domain
                    .iter()
                    .cloned()
                    .map(|s| s.to_string())
                    .reduce(|acc, e| format!("{e}.{acc}"))
                    .unwrap()
            );

            let is = exclude_trie.is_domain(&domain);
            #[cfg(debug_assertions)]
            println!("[+] {addr:?} exclude {is:?}");
            if is {
                tx_req.send((payload, addr)).await.expect("[E] tx_req send");
                continue;
            }

            let is = trie.is_domain(&domain);
            #[cfg(debug_assertions)]
            println!("[+] {addr:?} fake {is:?}");
            if is {
                let buf = fake_response(payload.as_ref(), end_offset);
                let _len = sock_local_c
                    .send_to(&buf, &addr)
                    .await
                    .expect("[E] sock_local send_to");

                #[cfg(debug_assertions)]
                println!("[+] {addr:?} send fake response({_len:?})");
            } else {
                tx_req.send((payload, addr)).await.expect("[E] tx_req send");
            }
        }
    });

    let mut buf = [0; 1024];
    loop {
        let (len, addr) = sock_local
            .recv_from(&mut buf)
            .await
            .expect("[E] sock_local recv_from");
        #[cfg(debug_assertions)]
        println!("[+] {addr:?} recv request({len:?})");

        tx.send((Payload::from(&buf[..len]), addr))
            .await
            .expect("[E] tx send");
    }
}
