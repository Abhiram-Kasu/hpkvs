use std::error::Error;
use std::ops::{Deref, DerefMut};
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
                                        stream
                                            .write(match parse_key_value(&data[1..]) {
                                                Some((key, value)) => {
                                                    println!("Read {key} : {value}");
                                                    handle_write(
                                                        key,
                                                        value,
                                                        kv_store.lock().await.deref_mut(),
                                                    );
                                                    "200\n".as_bytes()
                                                }
                                                None => {
                                                    "Failed to parse key and value\n".as_bytes()
                                                }
                                            })
                                            .await
                                            .expect("Failed to write");
                                    }
                                    Some("R") => match data.get(1..) {
                                        Some(rest) => {
                                            match handle_read(
                                                rest.trim(),
                                                kv_store.lock().await.deref(),
                                            ) {
                                                Some(item) => {
                                                    stream
                                                        .write(item.as_bytes())
                                                        .await
                                                        .expect("Failed to Read");
                                                }
                                                None => {
                                                    stream
                                                        .write(
                                                            "404 Failed to find item\n".as_bytes(),
                                                        )
                                                        .await
                                                        .expect("Failed to Write");
                                                }
                                            }
                                        }
                                        _ => (),
                                    },
                                    _ => {
                                        stream
                                            .write("Failed to parse".as_bytes())
                                            .await
                                            .expect("Failed to write error message");
                                        continue;
                                    }
                                }
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

fn handle_write(key: &str, value: &str, kv_store: &mut KVStore<String, String>) {
    kv_store.add_item(key.into(), value.into());
}

fn handle_read<'a>(key: &str, kv_store: &'a KVStore<String, String>) -> Option<&'a str> {
    kv_store.read_item(key.into()).map(|f| f.as_str())
}

fn parse_key_value(str: &str) -> Option<(&str, &str)> {
    println!("Parsing str: {str}");
    str.find(" : ")
        .map(|location| (str[0..location].trim(), str[location + 3..].trim()))
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
