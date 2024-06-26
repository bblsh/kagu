use std::fs::File;
use std::io::BufReader;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Stream, StreamConfig};
use crossbeam::channel::{Receiver, Sender};
use opus::{Decoder as OpusDecoder, Encoder};
use rodio::{Decoder, OutputStream, Sink};

use crate::audio_buffer_manager::AudioBufferManager;
use crate::audio_io::AudioIo;
use message::message::{Message, MessageHeader, MessageType};

#[derive(Debug)]
pub enum AudioManagerError {
    FailedToGetInputDevices,
    FailedToGetOutputDevices,
    FailedToGetSupportedConfigs,
    FailedToSetInputDevice,
    FailedToSetOutputDevice,
    FailedToCreateInputStream,
    FailedToCreateOutputStream,
    FailedToCreateEncoder,
    FailedToCreateDecoder,
    DeviceNotFound,
}

impl std::fmt::Debug for AudioManager {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "DEBUG WRITE NOT IMPLEMENTED")
    }
}

pub struct AudioManager {
    audio_out_sender: Sender<Message>,
    audio_in_receiver: Receiver<Message>,
    current_header: MessageHeader,
    input_stream: Option<Stream>,
    output_stream: Option<Stream>,
    audio_io: AudioIo,
}

impl AudioManager {
    pub fn new(
        audio_out_sender: Sender<Message>,
        audio_in_receiver: Receiver<Message>,
        current_header: MessageHeader,
    ) -> AudioManager {
        AudioManager {
            audio_out_sender,
            audio_in_receiver,
            current_header,
            input_stream: None,
            output_stream: None,
            audio_io: AudioIo::new(),
        }
    }

    pub fn set_header(&mut self, header: MessageHeader) {
        self.current_header = header;
    }

    pub fn start_recording(&mut self) -> Result<(), AudioManagerError> {
        let host = cpal::default_host();

        let input_device = match host.default_input_device() {
            Some(device) => device,
            None => return Err(AudioManagerError::FailedToGetInputDevices),
        };

        let config = StreamConfig {
            sample_rate: cpal::SampleRate(48000),
            channels: 1,
            buffer_size: cpal::BufferSize::Fixed(480),
        };

        let mut encoder =
            match Encoder::new(48000, opus::Channels::Stereo, opus::Application::Audio) {
                Ok(encoder) => encoder,
                Err(_) => return Err(AudioManagerError::FailedToCreateEncoder),
            };

        let audio_sender = self.audio_out_sender.clone();
        let mut header = self.current_header;

        let err_fn = move |err| {
            eprintln!("an error occurred on stream: {}", err);
        };

        let data_callback = move |data: &[f32], _: &_| {
            // Make mono recording into stereo
            let mut stereo_audio = [0.0; 960];
            for i in 0..480 {
                stereo_audio[i * 2] = data[i];
                stereo_audio[i * 2 + 1] = data[i];
            }

            if let Ok(bytes) = encoder.encode_vec_float(&stereo_audio, 480 * 8) {
                header.datetime = Some(chrono::Utc::now());

                let message = Message::from(MessageType::Audio((header, bytes)));

                let _ = audio_sender.send(message);
            }
        };

        if let Ok(stream) = input_device.build_input_stream(&config, data_callback, err_fn, None) {
            stream.play().unwrap();
            self.input_stream = Some(stream);
            Ok(())
        } else {
            Err(AudioManagerError::FailedToCreateInputStream)
        }
    }

    pub fn stop_recording(&mut self) {
        self.input_stream = None;
    }

    pub fn start_listening(&mut self) -> Result<(), AudioManagerError> {
        let host = cpal::default_host();

        let ouput_device = match host.default_output_device() {
            Some(device) => device,
            None => return Err(AudioManagerError::FailedToGetOutputDevices),
        };

        let config = StreamConfig {
            sample_rate: cpal::SampleRate(48000),
            channels: 2,
            buffer_size: cpal::BufferSize::Fixed(480),
        };

        let mut decoder = match OpusDecoder::new(48000, opus::Channels::Stereo) {
            Ok(decoder) => decoder,
            Err(_) => return Err(AudioManagerError::FailedToCreateDecoder),
        };

        let audio_receiver = self.audio_in_receiver.clone();
        let _header = self.current_header;

        let err_fn = move |err| {
            eprintln!("an error occurred on stream: {}", err);
        };

        let mut buffer_manager = AudioBufferManager::new();

        let data_callback = move |data: &mut [f32], _: &_| {
            // There's data to play back, so mix and play it back
            while let Ok(message) = audio_receiver.try_recv() {
                if let MessageType::Audio((header, audio)) = message.message {
                    // Volume manipulation may be able to be done here later on
                    let mut user_audio: [f32; 960] = [0.0; 960];
                    let _decoded_samples = decoder
                        .decode_float(audio.as_slice(), &mut user_audio, false)
                        .unwrap();

                    buffer_manager.buffer_data(header.user_id, user_audio);
                }
            }

            data[..960].copy_from_slice(&buffer_manager.get_output_data()[..960]);
        };

        if let Ok(stream) = ouput_device.build_output_stream(&config, data_callback, err_fn, None) {
            stream.play().unwrap();
            self.output_stream = Some(stream);
            Ok(())
        } else {
            Err(AudioManagerError::FailedToCreateOutputStream)
        }
    }

    pub fn stop_listening(&mut self) {
        self.output_stream = None;
    }

    pub fn get_audio_inputs(&self) -> Vec<String> {
        self.audio_io.get_input_devices().unwrap_or_default()
    }

    pub fn get_audio_outputs(&self) -> Vec<String> {
        self.audio_io.get_output_devices().unwrap_or_default()
    }

    pub fn set_audio_input(&mut self, input_name: String) {
        self.audio_io.set_input_device(input_name);
    }

    pub fn set_audio_output(&mut self, output_name: String) {
        self.audio_io.set_output_device(output_name);
    }

    pub fn play_audio_file(&self, file_path: String) {
        // Get a device to play audio back with
        let device = self.audio_io.get_output_device().unwrap();

        // Likely inefficient, but spawn a thread to play this sound
        let _audio_handle = std::thread::spawn(move || {
            let (_stream, stream_handle) = OutputStream::try_from_device(&device).unwrap();
            let sink = Sink::try_new(&stream_handle).unwrap();
            let file = BufReader::new(File::open(file_path).unwrap());
            let source = Decoder::new(file).unwrap();
            sink.append(source);
            sink.sleep_until_end();
        });
    }
}
