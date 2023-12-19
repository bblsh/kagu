use clap::Parser;
use client::client::Client;
use tui::app::{App, AppResult};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Ip address to use
    #[arg(short, long)]
    address: String,

    /// Port to listen on
    #[arg(short, long)]
    port: u16,

    /// Username to log in with
    #[arg(short, long)]
    username: String,
}

#[tokio::main]
async fn main() -> AppResult<()> {
    let args = Args::parse();

    let client = Client::new(args.address, args.port, args.username).await;
    client.run_client().await;

    loop {
        if client.get_user_id().await.is_some() {
            break;
        }
    }

    // Create an application.
    let mut app = App::new(client);
    app.run_app().await
}
