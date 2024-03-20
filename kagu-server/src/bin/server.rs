use std::path::PathBuf;

use clap::{ArgAction, Parser};
use server::new_server::NewServer;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port to listen on
    #[arg(short, long)]
    port: Option<u16>,

    /// If IPv6 should be used
    #[arg(long, action=ArgAction::SetTrue)]
    ipv6: Option<bool>,

    #[arg(short, long)]
    name: Option<String>,

    #[arg(short, long)]
    cert_dir: PathBuf,
}

fn main() {
    // Collect arguments
    let args = Args::parse();

    let port = args.port.unwrap_or(5000);

    let server_name = match args.name {
        Some(name) => name,
        None => String::from("KaguServer"),
    };

    let server = NewServer::new(server_name, port, args.ipv6, args.cert_dir);
    server.start_server();

    // Set up ctrl-c handler
    ctrlc::set_handler(move || {
        server.stop_server();
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    loop {
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}
