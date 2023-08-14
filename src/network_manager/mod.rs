use anyhow::Result;
use quinn::{ClientConfig, Connection, Endpoint, ServerConfig};
use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::mpsc::TryRecvError;
use std::sync::{mpsc, Arc, Mutex};

type Tx = mpsc::Sender<NetworkCommand>;
type Rx = mpsc::Receiver<NetworkCommand>;

#[derive(Debug)]
pub enum ConnectionCommand {
    StopReceiving,
}

pub enum ServerOrClient {
    Server,
    Client,
}
enum NetworkCommand {
    StopReceiving,
}

#[derive(Debug)]
pub struct NetworkManager {
    endpoint: Endpoint,
    connections: Arc<Mutex<HashMap<u32, Connection>>>,
    connection_senders: HashMap<u32, Tx>,
}

impl NetworkManager {
    pub async fn new(address: String, server_or_client: ServerOrClient) -> Endpoint {
        // Parse this address into a SocketAddr
        let address: SocketAddr = address.parse().unwrap();

        match server_or_client {
            // Configure this NetworkManager for a client
            ServerOrClient::Client => {
                let endpoint = make_client_endpoint("0.0.0.0:0".parse().unwrap()).unwrap();

                // // Here "localhost" should match the server cert (but this is ignored right now)
                // let connect = endpoint.connect(address, "localhost").unwrap();
                // let connection = connect.await;

                // let connection = match connection {
                //     Ok(conn) => conn,
                //     Err(ConnectionError::TimedOut) => {
                //         eprintln!("[NetworkManager] Connection timed out. Is the server IP and port correct?");
                //         std::process::exit(1);
                //     }
                //     Err(e) => {
                //         eprintln!("[NetworkManager] Error while connecting: {}", e);
                //         std::process::exit(1);
                //     }
                // };

                // let mut conns = HashMap::new();
                // conns.insert(0, connection);

                // let conns = Arc::new(Mutex::new(conns));

                endpoint
            }

            // Configure this NetworkManager for a server
            ServerOrClient::Server => {
                let (endpoint, _server_cert) = match make_server_endpoint(address) {
                    Ok((ep, cert)) => (ep, cert),
                    Err(_) => {
                        eprintln!("[server] failed to bind to address. exiting");
                        std::process::exit(1);
                    }
                };

                endpoint
            }
        }
    }

    pub fn disconnect(self: &mut Self) {
        if let Ok(mut connections) = self.connections.lock() {
            // Close each connection in our connections HashMap
            for (_id, conn) in connections.iter_mut() {
                println!("Closing connection...");
                conn.close(0u32.into(), b"done");
                println!("Closed connection");
            }

            // Clear our Connections because they've all been disconnected
            connections.clear();

            // Remove all of our connection senders, because we've disconnected
            for (_id, sender) in &self.connection_senders {
                match sender.send(NetworkCommand::StopReceiving) {
                    Ok(_) => (),
                    Err(_) => (),
                }
            }
        }
    }

    pub async fn send(self: &Self, buffer: &[u8]) {
        // Don't try to send anything if there's aren't any existing connections
        if let Ok(mut connections) = self.connections.lock() {
            if connections.is_empty() {
                println!("No connection to server made. Can't send a messaage yet.");
                return;
            }

            // Send this message to each connection (clients should only have one connection)
            for (_id, conn) in connections.iter_mut() {
                let (mut send, _recv) = conn.open_bi().await.unwrap();

                send.write_all(buffer).await.unwrap();
                send.finish().await.unwrap();
            }
        }
    }

    // Receive data from all existing connections
    pub async fn receive_data(
        self: &mut Self,
        handle_data: fn(Vec<u8>, connections: Arc<Mutex<HashMap<u32, Connection>>>),
    ) {
        // Listen for any connections
        while let Some(conn) = self.endpoint.accept().await {
            let connection = conn.await.unwrap();

            // Save connection somewhere, start transferring, receiving data, see DataTransfer tutorial.
            println!(
                "[server] incoming connection: addr={}",
                connection.remote_address()
            );

            let (tx, rx): (Tx, Rx) = mpsc::channel();

            self.connection_senders
                .insert(self.generate_connection_id(), tx);

            let connections = self.connections.clone();

            // Spawn a tokio thread to listen for data
            tokio::spawn(async move {
                loop {
                    // Listen for channel messages to stop listening on this channel
                    match rx.try_recv() {
                        Ok(command) => match command {
                            NetworkCommand::StopReceiving => {
                                break;
                            }
                        },
                        Err(TryRecvError::Empty) => (), // Do nothing here, nothing to receive yet
                        Err(TryRecvError::Disconnected) => {
                            eprintln!("No sender available to receive from");
                            drop(rx);
                            break;
                        }
                    }

                    let stream = connection.accept_bi().await;
                    let _stream = match stream {
                        Ok((_send_stream, mut read_stream)) => {
                            let message = read_stream.read_to_end(2048).await.unwrap();

                            handle_data(message, connections.clone());
                        }
                        Err(quinn::ConnectionError::ApplicationClosed { .. }) => {
                            println!("Connection closed");
                            break;
                        }
                        Err(e) => {
                            println!("Stream error: {}", e);
                            break;
                        }
                    };
                }
            });
        }
    }

    fn generate_connection_id(self: &Self) -> u32 {
        self.connection_senders.len() as u32
    }
}

// This is used for the client to blindly trust the server's cert
struct SkipServerVerification;

// This is used for the client to blindly trust the server's cert
impl SkipServerVerification {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

// This is used for the client to blindly trust the server's cert
impl rustls::client::ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        _server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::ServerCertVerified::assertion())
    }
}

// This is used for the client to blindly trust the server's cert
fn configure_client() -> ClientConfig {
    let crypto = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_custom_certificate_verifier(SkipServerVerification::new())
        .with_no_client_auth();

    ClientConfig::new(Arc::new(crypto))
}

// This is used for the client to blindly trust the server's cert
fn make_client_endpoint(bind_addr: SocketAddr) -> Result<Endpoint, Box<dyn Error>> {
    //let client_cfg = ClientConfig::with_native_roots();
    let mut endpoint = Endpoint::client(bind_addr)?;
    endpoint.set_default_client_config(configure_client());
    Ok(endpoint)
}

fn make_server_endpoint(bind_addr: SocketAddr) -> Result<(Endpoint, Vec<u8>), Box<dyn Error>> {
    let (server_config, server_cert) = configure_server()?;
    let endpoint = Endpoint::server(server_config, bind_addr)?;
    Ok((endpoint, server_cert))
}

fn configure_server() -> Result<(ServerConfig, Vec<u8>), Box<dyn Error>> {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    let cert_der = cert.serialize_der().unwrap();
    let priv_key = cert.serialize_private_key_der();
    let priv_key = rustls::PrivateKey(priv_key);
    let cert_chain = vec![rustls::Certificate(cert_der.clone())];

    let mut server_config = ServerConfig::with_single_cert(cert_chain, priv_key)?;
    let transport_config = Arc::get_mut(&mut server_config.transport).unwrap();
    transport_config.max_concurrent_uni_streams(0_u8.into());
    transport_config.keep_alive_interval(Some(std::time::Duration::from_secs(5)));

    Ok((server_config, cert_der))
}
