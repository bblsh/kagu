pub struct NewServer {
    server_name: String,
    port: u16,
    use_ipv6: Option<bool>,
}

impl NewServer {
    pub fn new(server_name: String, port: u16, use_ipv6: Option<bool>) -> NewServer {
        NewServer {
            server_name,
            port,
            use_ipv6,
        }
    }

    pub fn start_server(&self) {}
}
