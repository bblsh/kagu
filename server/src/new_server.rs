use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::path::{Path, PathBuf};

use crate::server_state::ServerState;
use network_manager::*;

use swiftlet_quic::endpoint::{Config, Endpoint};
use swiftlet_quic::EndpointHandler;

pub struct NewServer {
    server_name: String,
    port: u16,
    ipv6: Option<bool>,
    cert_dir: PathBuf,
}

impl NewServer {
    pub fn new(server_name: String, port: u16, ipv6: Option<bool>, cert_dir: PathBuf) -> NewServer {
        NewServer {
            server_name,
            port,
            ipv6,
            cert_dir,
        }
    }

    pub fn start_server(&self) {
        let bind_address = match self.ipv6 {
            Some(ipv6) => match ipv6 {
                true => SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, self.port, 0, 0)),
                false => SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, self.port)),
            },
            None => SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, self.port)),
        };

        let config = Config {
            idle_timeout_in_ms: 5000,
            reliable_stream_buffer: 65536,
            unreliable_stream_buffer: 65536,
            keep_alive_timeout: None,
            initial_main_recv_size: BUFFER_SIZE_PER_CONNECTION,
            main_recv_first_bytes: MESSAGE_HEADER_SIZE,
            initial_background_recv_size: BUFFER_SIZE_PER_CONNECTION,
            background_recv_first_bytes: MESSAGE_HEADER_SIZE,
            initial_rt_recv_size: 65536,
            rt_recv_first_bytes: 0,
        };

        let (cert, pkey) = self.get_pem_paths(&self.cert_dir);

        let mut server_endpoint = match Endpoint::new_server(
            bind_address,
            ALPN_NAME,
            cert.as_str(),
            pkey.as_str(),
            config,
        ) {
            Ok(endpoint) => endpoint,
            Err(e) => {
                println!("[server] failed to create server endpoint: {:?}", e);
                return;
            }
        };

        let mut server_state = ServerState::new(self.server_name.clone());

        let _server_handle = std::thread::spawn(move || {
            let mut endpoint_handler =
                EndpointHandler::new(&mut server_endpoint, &mut server_state);
            match endpoint_handler.run_event_loop(std::time::Duration::from_millis(5)) {
                Ok(_) => (),
                Err(e) => {
                    println!("[server]: event loop error: {:?}", e);
                }
            }
        });

        println!("[server] server started");
    }

    fn get_pem_paths(&self, cert_dir: &Path) -> (String, String) {
        let mut cert = cert_dir.to_str().unwrap().to_string();
        cert.push_str("/cert.pem");

        let mut pkey = cert_dir.to_str().unwrap().to_string();
        pkey.push_str("/pkey.pem");

        (cert, pkey)
    }
}
