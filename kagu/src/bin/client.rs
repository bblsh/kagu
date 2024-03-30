use std::{net::SocketAddr, path::PathBuf};

use clap::Parser;
use client::client::Client;
use tui::app::App;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Address to connect to.
    /// Must be in `127.0.0.1:5000` or `[::1]:5000` format
    #[arg(short, long)]
    address: SocketAddr,

    /// Username to log in with
    #[arg(short, long)]
    username: String,

    #[arg(short, long)]
    cert_dir: PathBuf,
}

fn main() {
    let args = Args::parse();

    let mut client = Client::new(args.address, args.username, args.cert_dir);
    client.run_client();

    let start_time = std::time::Instant::now();

    // Don't show the UI until we are connected
    loop {
        if client.is_connected() {
            break;
        } else {
            let current_time = std::time::Instant::now();
            if current_time - start_time > std::time::Duration::from_secs(2) {
                println!("Failed to connect. Exiting");
                std::process::exit(1);
            }
        }
    }

    // Create an application.
    let mut app = App::new(client);
    let _ = app.run_app();
}
