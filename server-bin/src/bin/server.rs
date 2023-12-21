use clap::{ArgAction, Parser};
use server::server::Server;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port to listen on
    #[arg(short, long)]
    port: u16,

    /// If IPv6 should be used
    #[arg(long, action=ArgAction::SetTrue)]
    ipv6: Option<bool>,
}

#[tokio::main]
async fn main() {
    // Collect arguments
    let args = Args::parse();

    match Server::new(args.port, args.ipv6).await {
        Ok(server) => server.run_server().await,
        Err(e) => eprintln!("Failed to start server: {}", e),
    };
}
