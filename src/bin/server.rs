use rustcord::server::Server;

#[tokio::main]
async fn main() {
    // Get the port to bind to
    let mut args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("server: a port for the server is needed");
        eprintln!("usage: server 5000");
        std::process::exit(1);
    }

    let mut address = String::from("0.0.0.0:");
    address.push_str(args.remove(1).as_str());

    let server = Server::new(address).await;
    server.run_server().await;
}
