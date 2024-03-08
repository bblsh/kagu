use message::message::{Message, MessageHeader, MessageType};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, Stream, StreamConfig};
use crossbeam::channel::{Receiver, Sender};
use opus::{Decoder, Encoder};

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

        if let Ok(stream) = input_device.build_input_stream(
            &config,
            move |data, _: &_| encode_and_send::<f32>(data, &mut encoder, &audio_sender, &header),
            err_fn,
            None,
        ) {
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
}

fn encode_and_send<T>(
    input: &[T],
    encoder: &mut Encoder,
    audio_out_sender: &Sender<Message>,
    message_header: &MessageHeader,
) where
    T: Sample,
    f32: FromSample<T>,
{
    let f32_samples = input
        .iter()
        .map(|f| f32::from_sample_(*f))
        .collect::<Vec<f32>>();

    let bytes = encoder
        .encode_vec_float(f32_samples.as_slice(), 1200)
        .unwrap();

    let message = Message::from(MessageType::Audio((*message_header, bytes)));

    let _ = audio_out_sender.send(message);
}
