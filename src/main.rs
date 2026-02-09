use std::error::Error;
use std::ops::Deref;
use std::sync::Arc;
use std::{borrow::Cow, str::Chars};

use hpkvs::KVStore;
use regex::bytes::Regex;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::Mutex,
};

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:8080";

    let parser = Parser::default();
    let kv_store = Arc::new(Mutex::new(KVStore::<String, String>::new()));
    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    println!("Server listening on {}", addr);

    loop {
        match listener.accept().await {
            Ok((mut stream, client_addr)) => {
                println!("New connection from: {}", client_addr);
                let parser = parser.clone();
                let kv_store = kv_store.clone();
                tokio::spawn(async move {
                    let mut buf = [0u8; 1024];
                    loop {
                        match stream.read(&mut buf[..]).await {
                            Ok(0) => {
                                println!("Client {} disconnected", client_addr);
                                break;
                            }
                            Ok(n) => {
                                dbg!(format!("Read {n} bytes"));
                                let data = String::from_utf8_lossy(&buf[..n]);

                                match data.get(0..1) {
                                    Some("W") => {
                                        if let Some((key, value)) = parse_key_value(&data) {
                                            handle_write(key, value, kv_store).await;
                                        }
                                    }
                                    Some("R") if data.len() > 1 => {
                                        if let Some(rest) = data.get(1..) {
                                            match handle_read(rest, kv_store.lock().await.deref()) {
                                                Some(item) => {}
                                                None => {}
                                            }
                                        }
                                    }
                                    _ => {
                                        stream
                                            .write("Failed to parse".as_bytes())
                                            .await
                                            .expect("Failed to write error message");
                                        continue;
                                    }
                                }
                                //replace escaped : with regular colon

                                // TODO: Handle key-value pair
                                println!("Key: {}, Value: {}", key, value);
                                // kv_store
                                //     .lock()
                                //     .await
                                //     .add_item(key.to_owned(), value.to_owned())
                                //     .await;
                            }
                            Err(e) => {
                                dbg!(e);
                                break;
                            }
                        }
                    }
                });
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
            }
        }
    }
}

fn handle_write(key: &str, value: &str, kv_store: Arc<Mutex<KVStore<String, String>>>) {}

fn handle_read<'a>(key: &str, kv_store: &'a KVStore<String, String>) -> Option<&'a str> {
    kv_store.read_item(key.into()).map(|f| f.as_str())
}

fn parse_key_value(str: &str) -> Option<(&str, &str)> {
    str.find(" : ")
        .map(|location| (&str[0..location], &str[location + 3..]))
}
#[derive(Clone, Debug)]
struct Parser {
    regex: Regex,
}

impl Default for Parser {
    fn default() -> Self {
        Self {
            regex: Regex::new("\\:").expect("Failed to build regex"),
        }
    }
}

impl Parser {
    pub fn clean<'a>(&self, data: &'a str) -> Cow<'a, [u8]> {
        self.regex.replace_all(data.as_bytes(), ":".as_bytes())
    }
}
