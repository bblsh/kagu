use std::collections::VecDeque;
use std::io::prelude::*;
use std::process::Command;
use std::process::Stdio;
use std::sync::mpsc::Receiver;
use std::{net::SocketAddr, path::PathBuf};

use message::message::{MessageHeader, MessageType};
use user::User;

use clap::Parser;
use client::client::Client;
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

enum BotCommand {
    Play(String),
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
    let mut current_voice_channel: Option<MessageHeader> = None;

    let mut command_queue: VecDeque<(u64, BotCommand)> = VecDeque::new();
    let mut num_commands: u64 = 0;

    println!("Logging in...");
    client.log_in();

    println!("Listening for messages...");

    loop {
        // Run until told to stop
        if should_exit(&recv) {
            exit(&mut client);
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
                        match message.0.as_str() {
                            "play" => {
                                client.send_mention_message(
                                    header.realm_id,
                                    header.channel_id,
                                    vec![(String::from("Now playing..."), None)],
                                );

                                // Don't join again if we are already in the channel
                                match current_voice_channel {
                                    None => {
                                        client.join_channel(
                                            header.realm_id,
                                            realms::realm::ChannelType::VoiceChannel,
                                            0,
                                        );
                                        current_voice_channel = Some(MessageHeader::new(
                                            user.as_ref().unwrap().get_id(),
                                            header.realm_id,
                                            0,
                                        ));

                                        let _ = &command_queue.push_back((
                                            num_commands,
                                            BotCommand::Play(String::from("url")),
                                        ));
                                        num_commands += 1;
                                    }
                                    Some(_channel) => {
                                        let _ = &command_queue.push_back((
                                            num_commands,
                                            BotCommand::Play(String::from("url")),
                                        ));
                                        num_commands += 1;
                                    }
                                }
                            }
                            "pause" => {
                                if client.is_broadcasting_audio() {
                                    client.pause_broadcasting();
                                }

                                client.send_mention_message(
                                    header.realm_id,
                                    header.channel_id,
                                    vec![(String::from("Paused."), None)],
                                );
                            }
                            "stop" => {
                                if client.is_broadcasting_audio() {
                                    client.stop_broadcasting();

                                    client.send_mention_message(
                                        header.realm_id,
                                        header.channel_id,
                                        vec![(String::from("Stopping playback. Goodbye."), None)],
                                    );

                                    client.hang_up(header.realm_id, 0);
                                    current_voice_channel = None;
                                }
                            }
                            "resume" => {
                                client.resume_broadcasting();

                                client.send_mention_message(
                                    header.realm_id,
                                    header.channel_id,
                                    vec![(String::from("Resuming playback."), None)],
                                );
                            }
                            _ => (),
                        }
                    }
                }
                MessageType::UserJoinedVoiceChannel(header) => {
                    if let Some(ref user) = user {
                        if header.user_id == user.get_id() {
                            current_voice_channel = Some(header);
                        }
                    }
                }
                _ => (),
            }
        }

        let mut commands_to_remove: Vec<u64> = Vec::new();
        for command in &command_queue {
            match &command.1 {
                BotCommand::Play(url) => {
                    if current_voice_channel.is_some() {
                        broadcast_audio(url.to_string(), &user, &mut client);
                        commands_to_remove.push(command.0);
                    }
                }
            }
        }

        for ctr in commands_to_remove {
            command_queue.retain(|id| id.0 != ctr);
        }

        // Leave a voice channel if audio isn't being broadcasted anymore
        if !client.is_broadcasting_audio() && current_voice_channel.is_some() {
            let channel = current_voice_channel.unwrap();
            client.hang_up(channel.realm_id, channel.channel_id);

            current_voice_channel = None;
        }

        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

fn broadcast_audio(url: String, user: &Option<User>, client: &mut Client) {
    if user.is_some() {
        println!("Fetching audio from {}", url);

        let output = Command::new("sh")
            .arg("-c")
            .arg("./stream.sh")
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to start command");

        let mut stdout = output.stdout.unwrap();
        let mut buffer = Vec::new();
        let _ = stdout.read_to_end(&mut buffer);

        println!("Fetched audio");

        // Create a decoder from the read audio samples
        let cursor = std::io::Cursor::new(buffer);
        let source = Decoder::new(cursor).unwrap();

        println!("Converting samples...");
        let converted_samples: VecDeque<f32> =
            source.convert_samples().map(|s: f32| s * 0.5).collect();

        println!("Encoding audio for playback...");
        client.broadcast_audio_buffer(Vec::from(converted_samples));

        println!("Playing back audio...");
    }
}

fn should_exit(recv: &Receiver<bool>) -> bool {
    // Run until told to stop
    matches!(recv.try_recv(), Ok(_stop))
}

fn exit(client: &mut Client) {
    println!();
    println!("Disconnecting...");
    client.disconnect();
    println!("Disconnected");
}
