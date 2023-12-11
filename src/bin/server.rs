use clap::Parser;
use kagu::server::server::Server;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Ip address to use
    #[arg(short, long, default_value = "0.0.0.0")]
    address: String,

    /// Port to listen on
    #[arg(short, long)]
    port: u16,
}

#[tokio::main]
async fn main() {
    // Collect arguments
    let args = Args::parse();

    let server = Server::new(args.address, args.port).await;
    server.run_server().await;
}
