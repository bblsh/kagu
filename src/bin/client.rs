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

    // Create an application.
    let mut app = App::new(client);
    app.run_app().await

    /*
    // Initialize the audio manager to handle all playback and recording
    let mut audio_manager = AudioManager::new();

    audio_manager.start_recording();
    audio_manager.start_listening();

    // Keep recording and sending until we command the program to stop
    loop {
        let mut input = String::new();
        let stdin = std::io::stdin();
        stdin.read_line(&mut input).unwrap();

        match input.as_str().trim_end() {
            "stop" => {
                println!("Stopping recording and exiting...");
                let _ = &audio_manager.stop_recording();
                let _ = &audio_manager.stop_listening();
                break;
            }
            _ => {
                println!("Not a valid command. Enter \"stop\" to stop recording");
            }
        }
    }
    */
}
