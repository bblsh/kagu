use anyhow::Result;
use core::fmt;
use quinn::{ClientConfig, Endpoint, ServerConfig};
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Debug)]
pub enum ConnectionCommand {
    StopReceiving,
}

pub enum ServerOrClient {
    Server,
    Client,
}

#[derive(Debug)]
pub enum NetworkManagerError {
    ParseError,
    NoAddressProvided,
    FailedToMakeClientEndpoint,
    FailedToMakeServerEndpoint,
}

impl fmt::Display for NetworkManagerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct NetworkManager {}

trait AppendPort {
    fn append_port(self, port: u16) -> String;
}

impl AppendPort for String {
    fn append_port(mut self, port: u16) -> String {
        self.push(':');
        self.push_str(port.to_string().as_str());
        self
    }
}

impl NetworkManager {
    pub async fn connect_endpoint(
        address: Option<SocketAddr>,
        use_ipv6: Option<bool>,
        port: u16,
        server_or_client: ServerOrClient,
    ) -> Result<Endpoint, NetworkManagerError> {
        match server_or_client {
            ServerOrClient::Server => {
                let mut address_t: SocketAddr =
                    String::from("0.0.0.0").append_port(port).parse().unwrap();

                // Check to see if we should serve with IPv6
                if use_ipv6.unwrap_or(false) {
                    address_t = String::from("[::]")
                        .append_port(port)
                        .as_str()
                        .parse()
                        .unwrap();
                }

                match make_server_endpoint(address_t) {
                    Ok((endpoint, _cert)) => Ok(endpoint),
                    Err(_) => Err(NetworkManagerError::FailedToMakeServerEndpoint),
                }
            }
            ServerOrClient::Client => {
                if let Some(a) = address {
                    match make_client_endpoint(match a.is_ipv6() {
                        true => SocketAddr::V6("[::]:0".parse().unwrap()),
                        false => SocketAddr::V4("0.0.0.0:0".parse().unwrap()),
                    }) {
                        Ok(endpoint) => Ok(endpoint),
                        Err(_) => Err(NetworkManagerError::FailedToMakeClientEndpoint),
                    }
                } else {
                    Err(NetworkManagerError::NoAddressProvided)
                }
            }
        }
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
