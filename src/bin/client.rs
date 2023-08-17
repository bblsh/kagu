use rustcord::client::Client;
use rustcord::tui::app::{App, AppResult};

#[tokio::main]
async fn main() -> AppResult<()> {
    // Get the IP and port of the server to connect to
    let mut args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("client: a username and ip/port of the server are needed");
        eprintln!("usage: client username 127.0.0.1:5000");
        std::process::exit(1);
    }

    let username = args.remove(1);
    let address = args.remove(1);
    let client = Client::new(address, username).await;
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
