use std::collections::VecDeque;
use std::io::prelude::*;
use std::process::Command;
use std::process::Stdio;
use std::sync::mpsc::Receiver;
use std::time::Instant;
use std::{net::SocketAddr, path::PathBuf};

use message::message::MessageType;
use user::User;

use clap::Parser;
use client::client::Client;
use opus::Encoder;
use rodio::{Decoder, Source};

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

    let mut user: Option<User> = None;

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
                MessageType::LoginSuccess(our_user) => {
                    user = Some(our_user.clone());
                    client.set_user(our_user);
                    println!("Logged in");
                }
                MessageType::Text((header, mut chunks)) => {
                    if let Some(message) = chunks.pop() {
                        if message.0 == *"play" {
                            client.send_mention_message(
                                header.realm_id,
                                header.channel_id,
                                vec![(String::from("Now playing..."), None)],
                            );

                            client.join_channel(
                                header.realm_id,
                                realms::realm::ChannelType::VoiceChannel,
                                0,
                            );
                        }
                    }
                }
                MessageType::UserJoinedVoiceChannel(header) => {
                    if let Some(ref user) = user {
                        if header.user_id == user.get_id() {
                            println!("It's go time");
                            // We are in the voice channel. Start broadcasting the audio
                            println!("Fetching song...");
                            let output = Command::new("sh")
                                .arg("-c")
                                .arg("./stream.sh")
                                .stdout(Stdio::piped())
                                .spawn()
                                .expect("Failed to start command");

                            let mut stdout = output.stdout.unwrap();
                            let mut buffer = Vec::new();
                            let _ = stdout.read_to_end(&mut buffer);

                            println!("LEN: {}", buffer.len());

                            println!("Fetched song. Playing back...");

                            // Create a decoder from the read audio samples
                            let cursor = std::io::Cursor::new(buffer);
                            let source = Decoder::new(cursor).unwrap();

                            println!("Converting samples...");
                            let samples = source.convert_samples::<f32>();
                            let mut converted_samples = VecDeque::new();
                            for sample in samples {
                                converted_samples.push_back(sample);
                            }

                            println!("Building encoder...");
                            let mut encoder = Encoder::new(
                                48000,
                                opus::Channels::Stereo,
                                opus::Application::Audio,
                            )
                            .unwrap();
                            println!("Opus encoder built");

                            println!("Playing back audio...");

                            let mut done_playing = false;

                            // Pre-encode and buffer the audio
                            let mut buffers_to_send: VecDeque<Vec<u8>> = VecDeque::new();
                            loop {
                                let mut buffer = Vec::with_capacity(960);

                                for _ in 0..480 {
                                    buffer.push(converted_samples.pop_front().unwrap_or(0.0));
                                    buffer.push(converted_samples.pop_front().unwrap_or(0.0));
                                }

                                let bytes = encoder
                                    .encode_vec_float(buffer.as_slice(), 480 * 8)
                                    .unwrap();

                                buffers_to_send.push_back(bytes);

                                if converted_samples.is_empty() {
                                    break;
                                }
                            }

                            let mut time_to_send =
                                Instant::now() + std::time::Duration::from_millis(10);

                            // Send the audio
                            loop {
                                // Run until told to stop
                                if should_exit(&recv) {
                                    // Leave the voice channel since we are done playing
                                    client.hang_up(header.realm_id, header.channel_id);
                                    println!("Stopped playing audio");
                                    break;
                                }

                                if Instant::now() >= time_to_send {
                                    match buffers_to_send.pop_front() {
                                        Some(buffer) => client.send_audio_frame(header, buffer),
                                        None => done_playing = true,
                                    }

                                    time_to_send =
                                        Instant::now() + std::time::Duration::from_millis(10);
                                }

                                if done_playing {
                                    println!("Done!");
                                    break;
                                }
                            }

                            // Leave the voice channel since we are done playing
                            client.hang_up(header.realm_id, header.channel_id);
                        }
                    }
                }
                _ => (),
            }
        }

        // Run until told to stop
        if should_exit(&recv) {
            println!("Disconnecting...");
            client.disconnect();
            println!("Disconnected");
            break;
        }
    }
}

fn should_exit(recv: &Receiver<bool>) -> bool {
    // Run until told to stop
    if let Ok(_stop) = recv.try_recv() {
        true
    } else {
        false
    }
}
