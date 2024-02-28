use std::{net::SocketAddr, path::PathBuf};

use clap::Parser;
use client::new_client::NewClient;
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

    let client = NewClient::new(args.address, args.username, args.cert_dir);
    client.run_client();

    std::thread::sleep(std::time::Duration::from_millis(500));

    // client.log_in(String::from("TestHello"));

    // //std::thread::sleep(std::time::Duration::from_millis(500));

    // loop {
    //     std::thread::sleep(std::time::Duration::from_secs(1));
    //     for message in client.get_new_messages() {
    //         println!("{:?}", message);
    //     }
    // }

    // Create an application.
    let mut app = App::new(client);
    let _ = app.run_app();
}
