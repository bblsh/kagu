use std::{net::SocketAddr, path::PathBuf};

use message::message::MessageType;

use clap::Parser;
use client::client::Client;

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

    let (send, recv): (
        std::sync::mpsc::Sender<bool>,
        std::sync::mpsc::Receiver<bool>,
    ) = std::sync::mpsc::channel();

    // Set up ctrl-c handler
    ctrlc::set_handler(move || {
        let _ = send.send(true);
    })
    .expect("Error setting Ctrl-C handler");

    // Don't anything until we are connected
    let start_time = std::time::Instant::now();
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

    println!("Logging in...");
    client.log_in();

    println!("Listening for messages...");

    loop {
        // Run until told to stop
        if let Ok(_stop) = recv.try_recv() {
            println!("Disconnecting...");
            client.disconnect();
            println!("Disconnected");
            break;
        }

        for message in client.get_new_messages() {
            match message.message {
                MessageType::LoginSuccess(user) => {
                    client.set_user(user);
                    println!("Logged in");
                }
                _ => (),
            }
        }
    }
}
