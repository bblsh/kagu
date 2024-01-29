use crate::MAX_DATAGRAM_SIZE;

pub struct QuicheConfig {}

impl QuicheConfig {
    pub fn create_quiche_config(
        alpn: &[u8],
        cert_path: &str,
        pkey_path_option: Option<&str>,
        dgram_queue_len_option: Option<usize>,
    ) -> Result<quiche::Config, quiche::Error> {
        let mut config = match quiche::Config::new(quiche::PROTOCOL_VERSION) {
            Ok(cfg) => {
                cfg // A quiche Config with default values
            }
            Err(err) => {
                return Err(err);
            }
        };

        if let Some(pkey_path) = pkey_path_option {
            config.load_cert_chain_from_pem_file(cert_path)?;

            config.load_priv_key_from_pem_file(pkey_path)?;
            config.verify_peer(false);
            config.set_initial_max_streams_bidi(1); // Should be 1 here for server?

            // Enable datagram frames for unreliable realtime data to be sent
            //let dgram_queue_len = MAX_DATAGRAM_SIZE * (MAX_SERVER_CONNS as usize) * 2;
            config.log_keys();
        } else {
            config.load_verify_locations_from_file(cert_path)?; // Temporary solution for client to verify certificate

            config.verify_peer(true);
            config.set_initial_max_streams_bidi(1);

            //let dgram_queue_len = MAX_DATAGRAM_SIZE * (MAX_SERVER_CONNS as usize) || MAX_DATAGRAM_SIZE;
        }

        // Enable datagram frames for unreliable realtime data to be sent
        if let Some(dgram_queue_len) = dgram_queue_len_option {
            config.enable_dgram(true, dgram_queue_len * 10, dgram_queue_len);
        }

        let _ = config.set_application_protos(&[alpn]);

        config.set_max_idle_timeout(5000); // Use a timeout of infinite when this line is commented out

        config.set_max_recv_udp_payload_size(MAX_DATAGRAM_SIZE);
        config.set_max_send_udp_payload_size(MAX_DATAGRAM_SIZE);
        config.set_initial_max_data(16_777_216); // 16 MiB
        config.set_initial_max_stream_data_bidi_local(2_097_152); // 2 MiB
        config.set_initial_max_stream_data_bidi_remote(2_097_152); // 2 MiB

        config.set_initial_max_streams_uni(3);
        config.set_initial_max_stream_data_uni(2_097_152); // 2 MiB

        config.set_disable_active_migration(true); // Temporary

        Ok(config)
    }
}
