mod trie;

use clap::Parser;
use std::{fs::read_to_string, net::SocketAddr, sync::Arc};
use tokio::{net::UdpSocket, sync::mpsc};

use crate::trie::Trie;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Domain
    #[arg(short, long, default_value = "deploy/domain.conf")]
    domain: String,
}

const RESPONSE_START: &[u8] = &[0x81, 0x80, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01];
const RESPONSE_END: &[u8] = &[
    0xc0, 0x0c, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x01, 0xf4, 0x00, 0x04, 0x01, 0x02, 0x03, 0x04,
    0x00, 0x00, 0x29, 0x05, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

#[tokio::main]
async fn main() {
    let Args { domain } = Args::parse();
    println!("domain: {}", domain);

    let mut trie = Trie::new();
    trie.init_doamin(&domain);

    let sock_r = Arc::new(UdpSocket::bind("0.0.0.0:53").await.unwrap());
    let sock_s = sock_r.clone();
    println!("bind: 53");

    let sock_remote = Arc::new(UdpSocket::bind("0.0.0.0:58888").await.unwrap());
    let sock_remote_addr = "223.5.5.5:53".parse::<SocketAddr>().unwrap();

    let (tx, mut rx) = mpsc::channel::<(Vec<u8>, SocketAddr)>(3);

    tokio::spawn(async move {
        let mut rbuf = [0; 1024];
        let trie = Arc::new(trie);
        while let Some((buf, addr)) = rx.recv().await {
            let trie = trie.clone();
            let (is, end_offset) = trie.is_domain(&buf);
            if is {
                let buf = response(&buf, end_offset);
                let _len = sock_s.send_to(&buf, &addr).await.unwrap();
                #[cfg(debug_assertions)]
                dbg!(_len);
            } else {
                sock_remote.connect(sock_remote_addr).await.unwrap();
                sock_remote.send(&buf).await.unwrap();
                let len = sock_remote.recv(&mut rbuf).await.unwrap();

                let _len = sock_s.send_to(&rbuf[..len], &addr).await.unwrap();
                #[cfg(debug_assertions)]
                dbg!(_len);
            }
        }
    });

    let mut buf = [0; 1024];
    loop {
        let (len, addr) = sock_r.recv_from(&mut buf).await.unwrap();
        #[cfg(debug_assertions)]
        dbg!(len, addr);

        tx.send((buf[..len].to_vec(), addr)).await.unwrap();
    }
}

fn response(buf: &[u8], end_offset: usize) -> Vec<u8> {
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

impl Trie {
    fn init_doamin(&mut self, domain_path: &str) {
        for line in read_to_string(domain_path).unwrap().lines() {
            let mut d: Vec<_> = line.trim().split(".").collect();
            d.reverse();
            self.insert(&d);
        }
    }

    fn parse_domain_name<'a>(&'a self, message: &'a [u8], offset: usize) -> (Vec<&[u8]>, usize) {
        let mut domain: Vec<&[u8]> = Vec::new();
        let mut current_offset = offset;

        loop {
            let label_length = message[current_offset] as usize;

            if label_length == 0 {
                break;
            }

            // Regular label
            let label = &message[current_offset + 1..current_offset + 1 + label_length];
            domain.push(label);

            // Move to the next label
            current_offset += label_length + 1;
        }

        (domain, current_offset)
    }

    pub fn is_domain(&self, buf: &[u8]) -> (bool, usize) {
        let (domain, end_offset) = self.parse_domain_name(buf, 12);
        #[cfg(debug_assertions)]
        dbg!(end_offset);

        let mut domain: Vec<&str> = domain
            .iter()
            .map(|&bytes| std::str::from_utf8(bytes).unwrap())
            .collect();
        domain.reverse();
        #[cfg(debug_assertions)]
        dbg!(&domain);

        if self.starts_with(&domain) {
            #[cfg(debug_assertions)]
            dbg!(true);

            (true, end_offset)
        } else {
            #[cfg(debug_assertions)]
            dbg!(false);

            (false, end_offset)
        }
    }
}
