use core::mem::MaybeUninit;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Host, Sample, Stream, StreamConfig, SupportedStreamConfigRange};
use quinn::{Connection, Endpoint};
use ringbuf::{Consumer, SharedRb};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc::{channel, Receiver, Sender};

use crate::message::{Message, MessageHeader, MessageType};
use crate::types::UserIdSize;

pub enum AudioManagerError {
    FailedToGetInputDevices,
    FailedToGetOutputDevices,
    FailedToGetSupportedConfigs,
    FailedToSetInputDevice,
    FailedToSetOutputDevice,
    DeviceNotFound,
}

#[derive(Debug)]
pub enum AudioCommand {
    StartListening,
    StopListening,
    StartRecording,
    StopRecording,
}

impl std::fmt::Debug for AudioManager {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", "DEBUG WRITE NOT IMPLEMENTED")
    }
}

pub struct AudioManager {
    endpoint: Option<Endpoint>,
    connection: Option<Connection>,
    user_id: Option<UserIdSize>,
    audio_receiver: Option<Receiver<(UserIdSize, Vec<u8>)>>,
    host: Host,
    pub config: StreamConfig,
    pub input_device: String,
    pub output_device: String,
    pub record_command_sender: Option<Sender<AudioCommand>>,
    pub record_command_receiver: Option<Receiver<AudioCommand>>,
    pub play_command_sender: Option<Sender<AudioCommand>>,
    pub play_command_receiver: Option<Receiver<AudioCommand>>,
    input_stream: Option<Stream>,
    output_stream: Option<Stream>,
}

impl AudioManager {
    /// Generates and returns a new `AudioManager` with default values
    pub fn new() -> AudioManager {
        let host = cpal::default_host();

        let input_device = host
            .default_input_device()
            .expect("Couldn't get default input device");

        let output_device = host
            .default_output_device()
            .expect("Couldn't get the default output device");

        let config = StreamConfig {
            sample_rate: cpal::SampleRate(44100),
            channels: 1,
            buffer_size: cpal::BufferSize::Fixed(4096),
        };

        AudioManager {
            user_id: None,
            endpoint: None,
            connection: None,
            audio_receiver: None,
            input_device: input_device.name().unwrap(),
            output_device: output_device.name().unwrap(),
            host: host,
            config: config,
            record_command_sender: None,
            record_command_receiver: None,
            play_command_sender: None,
            play_command_receiver: None,
            input_stream: None,
            output_stream: None,
        }
    }

    pub fn endpoint(mut self, endpoint: Endpoint) -> Self {
        self.endpoint = Some(endpoint);
        self
    }

    pub fn connection(mut self, connection: Connection) -> Self {
        self.connection = Some(connection);
        self
    }

    pub fn user_id(mut self, user_id: UserIdSize) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn audio_receiver(mut self, receiver: Receiver<(UserIdSize, Vec<u8>)>) -> Self {
        self.audio_receiver = Some(receiver);
        self
    }

    pub fn set_user_id(&mut self, user_id: UserIdSize) {
        self.user_id = Some(user_id);
    }

    pub fn set_config(&mut self, sample_rate: u32, channels: u16, buffer_size: u32) {
        self.config = StreamConfig {
            sample_rate: cpal::SampleRate(sample_rate),
            channels: channels,
            buffer_size: cpal::BufferSize::Fixed(buffer_size),
        };
    }

    pub fn get_input_devices(&self) -> Vec<String> {
        let input_devices: Vec<String> = match self.host.input_devices() {
            Ok(devices) => devices
                .into_iter()
                .map(|device| match device.name() {
                    Ok(name) => name,
                    Err(_) => String::from("Unknown device name"),
                })
                .collect(),
            Err(e) => {
                eprintln!("[audio_manager] error getting input devices: {}", e);
                Vec::new()
            }
        };

        input_devices
    }

    pub fn set_input_device(&mut self, device_name: String) -> Result<(), AudioManagerError> {
        match self.host.input_devices() {
            Ok(mut devs) => {
                match devs.find(|x| x.name().map(|y| y == device_name).unwrap_or(false)) {
                    Some(device) => {
                        self.input_device =
                            device.name().unwrap_or(String::from("Unknown device name"));
                        Ok(())
                    }
                    None => Err(AudioManagerError::FailedToSetInputDevice),
                }
            }
            Err(_) => Err(AudioManagerError::FailedToSetInputDevice),
        }
    }

    pub fn get_output_devices(&self) -> Vec<String> {
        let output_devices: Vec<String> = match self.host.output_devices() {
            Ok(devices) => devices
                .into_iter()
                .map(|device| match device.name() {
                    Ok(name) => name,
                    Err(_) => String::from("Unknown device name"),
                })
                .collect(),
            Err(e) => {
                eprintln!("[audio_manager] error getting output devices: {}", e);
                Vec::new()
            }
        };

        output_devices
    }

    pub fn set_output_device(&mut self, device_name: String) -> Result<(), AudioManagerError> {
        match self.host.output_devices() {
            Ok(mut devs) => {
                match devs.find(|x| x.name().map(|y| y == device_name).unwrap_or(false)) {
                    Some(device) => {
                        self.output_device =
                            device.name().unwrap_or(String::from("Unknown device name"));
                        Ok(())
                    }
                    None => Err(AudioManagerError::FailedToSetOutputDevice),
                }
            }
            Err(_) => Err(AudioManagerError::FailedToSetOutputDevice),
        }
    }

    pub async fn start_recording(self: &mut Self) {
        // Generate a sender and receiver to start or stop recording
        let (tx, _rx): (Sender<AudioCommand>, Receiver<AudioCommand>) = channel(100);

        // Keep the message sender, and give the receiver to the recording thread
        self.record_command_sender = Some(tx);
        self.record();
    }

    pub async fn start_listening(self: &mut Self) {
        self.start_listen_thread().await;
    }

    async fn start_listen_thread(&mut self) {
        // Don't start a playback thread if we're already started up
        if self.output_stream.is_some() {
            return;
        }

        let err_fn = |err| eprintln!("an error occurred on the output audio stream: {}", err);
        let host = cpal::default_host();
        let output_device = host
            .default_output_device()
            .expect("no output device available");

        // CHANGE THE INPUT AND OUTPUT DEVICES TO USE STRINGS TO ID THEM INSTEAD OF SAVING THE DEVICE

        let config = self.config.clone();

        let buffer_size = 4096 * 10;
        let ring = SharedRb::<f32, Vec<_>>::new(buffer_size);
        let (mut producer, mut consumer) = ring.split();

        let output_stream = output_device
            .build_output_stream(
                &config,
                move |data, oci: &cpal::OutputCallbackInfo| {
                    AudioManager::play_audio(data, oci, &mut consumer)
                },
                err_fn,
                None,
            )
            .unwrap();

        output_stream.play().unwrap();

        self.output_stream = Some(output_stream);

        if self.audio_receiver.is_some() {
            let mut receiver = self.audio_receiver.take().unwrap();
            let _handle = tokio::spawn(async move {
                // This buffer holds audio from each user
                let mut audio_buffers: HashMap<UserIdSize, VecDeque<f32>> = HashMap::new();

                // Timer for mixing audio
                // Mix all audio captured within 20ms and release that to the playback buffer
                let mut time = Instant::now();

                loop {
                    match receiver.recv().await {
                        Some((user_id, audio)) => {
                            let mut audio: VecDeque<f32> = audio
                                .chunks_exact(4)
                                .map(|x| f32::from_le_bytes(x.try_into().unwrap()))
                                .collect();

                            if let Some(buffer) = audio_buffers.get_mut(&user_id) {
                                for _ in 0..audio.len() {
                                    buffer.push_back(audio.pop_front().unwrap());
                                }
                            }
                            // User doesn't exist, so make a buffer for that user
                            else {
                                audio_buffers.insert(user_id, audio);
                            }

                            // Now that we have this audio in memory, check to see if 20ms have passed
                            // If 20ms have passed, add all of the audio from each buffer and push
                            // the added buffer to the output buffer for playback
                            if time.elapsed().as_millis() > 60 {
                                // Place to hold mixed audio
                                let mut mixed: VecDeque<f32> = VecDeque::new();

                                // Let's first get the size of the longest buffer
                                let mut max_len: u32 = 0;
                                for buffer in audio_buffers.iter() {
                                    if buffer.1.len() as u32 > max_len {
                                        max_len = buffer.1.len() as u32;
                                    }
                                }

                                // Now that we have the max buffer length, push that many
                                // values to our mixed audio
                                for _ in 0..max_len {
                                    mixed.push_back(0.0);
                                }

                                // We have a buffer ready to fit all mixed audio
                                // Let's add it all up now
                                for buffer in audio_buffers.iter_mut() {
                                    for i in 0..buffer.1.len() {
                                        mixed[i] = mixed[i] + buffer.1[i];
                                    }

                                    // Now that we've added this, clear the buffer
                                    buffer.1.clear();
                                }

                                // Now that we have a mixed buffer, push that for playback
                                for _ in 0..mixed.len() {
                                    let _ = producer.push(mixed.pop_front().unwrap());
                                }

                                // Now reset the timer
                                time = Instant::now();
                            }
                        }
                        None => (),
                    };
                }
            });
        }
    }

    fn play_audio(
        data: &mut [f32],
        _: &cpal::OutputCallbackInfo,
        consumer: &mut Consumer<f32, Arc<SharedRb<f32, Vec<MaybeUninit<f32>>>>>,
    ) {
        for sample in data {
            *sample = match consumer.pop() {
                Some(s) => s,
                None => 0.0,
            };
        }
    }

    fn record(&mut self) {
        // First we have to get our host
        let host = &self.host;

        // Then get an input device
        let input_device = host
            .default_input_device()
            .expect("Couldn't get default input device");

        // Define an error function to use if recording fails
        let err_fn = move |err| {
            eprintln!("an error occurred on stream: {}", err);
        };

        let config = self.config.clone();

        match self.connection.clone() {
            Some(conn) => {
                let stream = input_device
                    .build_input_stream(
                        &config,
                        move |data, _: &_| pack_and_send_data::<f32>(data, conn.clone()),
                        err_fn,
                        None,
                    )
                    .unwrap();

                stream.play().unwrap();

                self.input_stream = Some(stream);
            }
            None => (),
        };
    }

    pub async fn disconnect(&mut self) {
        self.stop_recording().await;
        //self.stop_listening().await;
    }

    pub async fn stop_listening(self: &Self) {
        match &self.play_command_sender {
            Some(sender) => match sender.send(AudioCommand::StopListening).await {
                Ok(_) => (),
                Err(e) => println!("[audio_manager] error sending in stop_listening(): {}", e),
            },
            None => (),
        }
    }

    pub async fn stop_recording(&mut self) {
        self.input_stream = None;
    }

    pub fn get_supported_input_configs(
        &self,
        device_name: String,
    ) -> Result<
        (
            Vec<SupportedStreamConfigRange>,
            Vec<SupportedStreamConfigRange>,
        ),
        AudioManagerError,
    > {
        let mut input_configs: Vec<SupportedStreamConfigRange> = Vec::new();
        let mut output_configs: Vec<SupportedStreamConfigRange> = Vec::new();

        match self.host.input_devices() {
            Ok(mut devices) => {
                let mut configs: Vec<SupportedStreamConfigRange> = devices
                    .find(|x| x.name().map(|y| y == device_name).unwrap_or(false))
                    .unwrap()
                    .supported_input_configs()
                    .unwrap()
                    .into_iter()
                    .map(|config| config)
                    .collect();

                input_configs.append(&mut configs);
            }
            Err(_) => return Err(AudioManagerError::FailedToGetInputDevices),
        }

        match self.host.output_devices() {
            Ok(mut devices) => {
                let mut configs: Vec<SupportedStreamConfigRange> = devices
                    .find(|x| x.name().map(|y| y == device_name).unwrap_or(false))
                    .unwrap()
                    .supported_output_configs()
                    .unwrap()
                    .into_iter()
                    .map(|config| config)
                    .collect();

                output_configs.append(&mut configs);
            }
            Err(_) => return Err(AudioManagerError::FailedToGetOutputDevices),
        }

        Ok((input_configs, output_configs))

        // Example:
        // match self.input_device.supported_input_configs() {
        //     Ok(c) => {
        //         for config in c.into_iter() {
        //             println!("---------------------------------------");
        //             println!("Channels: {}", config.channels());
        //             println!("Max sample rate: {}", config.max_sample_rate().0);
        //             println!("Min sample rate: {}", config.min_sample_rate().0);
        //             match config.sample_format() {
        //                 _ => println!("Using format {}", config.sample_format()),
        //             };
        //         }
        //     }
        //     Err(e) => println!("Couldn't get supported configs: {}", e),
        // };
    }

    async fn send(buffer: &[u8], connection: Connection) {
        match connection.open_bi().await {
            // Ignore all errors because lazy
            Ok((mut send, _recv)) => match send.write_all(buffer).await {
                Ok(_) => match send.finish().await {
                    Ok(_) => (),
                    Err(e) => eprintln!("[audio_manager] error in send.finish(): {} ", e),
                },
                Err(e) => eprintln!("[audio_manager] error in send.write_all(): {} ", e),
            },
            Err(e) => eprintln!("[audio_manager] error in connection.open_bi(): {} ", e),
        }
    }
}

fn pack_and_send_data<T>(input: &[T], connection: Connection)
where
    T: Sample,
    f32: FromSample<T>,
{
    let mut audio: Vec<u8> = Vec::new();
    for &sample in input.iter() {
        //     // Samples are in memory as f32s, but we want each sample as a u16 to match the RTP spec
        //     // let sample_i16 = (f32::from_sample(sample) * 32767.0) as i16;
        //     // let sample_u16 = sample_i16 as u16;

        //     // audio.extend_from_slice(&sample_u16.to_be_bytes());
        audio.extend_from_slice(&f32::from_sample(sample).to_le_bytes());
    }

    // // Pack the data
    // let message = Message::from(MessageType::Audio(audio));
    let message = Message::from(MessageType::Audio((MessageHeader::new(0, 0, 0), audio)));

    // Create the runtime
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Execute the future, blocking the current thread until completion
    rt.block_on(async move {
        AudioManager::send(message.into_vec_u8().unwrap().as_slice(), connection).await;
    });
}
