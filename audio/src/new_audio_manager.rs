use message::message::{Message, MessageHeader, MessageType};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Stream, StreamConfig};
use crossbeam::channel::{Receiver, Sender};
use opus::{Decoder, Encoder};

use crate::audio_buffer_manager::AudioBufferManager;

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

impl std::fmt::Debug for NewAudioManager {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "DEBUG WRITE NOT IMPLEMENTED")
    }
}

pub struct NewAudioManager {
    audio_out_sender: Sender<Message>,
    audio_in_receiver: Receiver<Message>,
    current_header: MessageHeader,
    input_stream: Option<Stream>,
    output_stream: Option<Stream>,
}

impl NewAudioManager {
    pub fn new(
        audio_out_sender: Sender<Message>,
        audio_in_receiver: Receiver<Message>,
        current_header: MessageHeader,
    ) -> NewAudioManager {
        NewAudioManager {
            audio_out_sender,
            audio_in_receiver,
            current_header,
            input_stream: None,
            output_stream: None,
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

        let mut encoder = match Encoder::new(48000, opus::Channels::Mono, opus::Application::Audio)
        {
            Ok(encoder) => encoder,
            Err(_) => return Err(AudioManagerError::FailedToCreateEncoder),
        };

        let audio_sender = self.audio_out_sender.clone();
        let header = self.current_header;

        let err_fn = move |err| {
            eprintln!("an error occurred on stream: {}", err);
        };

        let data_callback = move |data: &[f32], _: &_| {
            if let Ok(bytes) = encoder.encode_vec_float(data, 480 * 8) {
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
            channels: 1,
            buffer_size: cpal::BufferSize::Fixed(480),
        };

        let mut decoder = match Decoder::new(48000, opus::Channels::Mono) {
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
            data.fill(0.0);

            // There's data to play back, so mix and play it back
            while let Ok(message) = audio_receiver.try_recv() {
                if let MessageType::Audio((header, audio)) = message.message {
                    // Volume manipulation may be able to be done here later on
                    let mut user_audio: [f32; 480] = [0.0; 480];
                    let _decoded_samples = decoder
                        .decode_float(audio.as_slice(), &mut user_audio, false)
                        .unwrap();

                    buffer_manager.buffer_data(header.user_id, user_audio);
                }
            }

            data[..480].copy_from_slice(&buffer_manager.get_output_data()[..480]);
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
}
