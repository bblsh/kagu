use std::net::SocketAddr;

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
}

fn main() {
    let args = Args::parse();

    // match NewClient::new(args.address, args.username) {
    //     Ok(client) => {
    //         client.run_client().await;

    //         loop {
    //             if client.get_user_id().await.is_some() {
    //                 break;
    //             }
    //         }

    //         // Create an application.
    //         let mut app = App::new(client);
    //         let _ = app.run_app().await;
    //     }
    //     Err(e) => eprintln!("Failed to start client: {}", e),
    // };
}
