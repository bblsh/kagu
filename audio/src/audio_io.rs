use cpal::{
    traits::{DeviceTrait, HostTrait},
    Device, Host,
};

#[derive(Debug)]
pub enum AudioIoError {
    FailedToGetDefaultInput,
    FailedToGetDefaultOutput,
    FailedToGetInputDevices,
    FailedToGetOutputDevices,
    FailedToGetInputDevice,
    FailedToGetOutputDevice,
    FailedToFindInputDevice,
    FailedToFindOutputDevice,
}

pub struct AudioIo {
    input_device: Option<String>,
    output_device: Option<String>,
    host: Host,
}

impl AudioIo {
    pub fn new() -> AudioIo {
        let host = cpal::default_host();

        AudioIo {
            input_device: None,
            output_device: None,
            host,
        }
    }

    pub fn get_input_devices(&self) -> Result<Vec<String>, AudioIoError> {
        match self.host.input_devices() {
            Ok(inputs) => Ok(inputs
                .map(|d| d.name().unwrap_or(String::from("Unknown")))
                .collect()),
            Err(_) => Err(AudioIoError::FailedToGetInputDevices),
        }
    }

    pub fn get_output_devices(&self) -> Result<Vec<String>, AudioIoError> {
        match self.host.output_devices() {
            Ok(outputs) => Ok(outputs
                .map(|d| d.name().unwrap_or(String::from("Unknown")))
                .collect()),
            Err(_) => Err(AudioIoError::FailedToGetOutputDevices),
        }
    }

    pub fn get_input_device(&self) -> Result<Device, AudioIoError> {
        match &self.input_device {
            Some(device_name) => match self.host.input_devices() {
                Ok(mut devices) => {
                    match devices.find(|d| {
                        d.name().unwrap_or(String::from("Unknown")) == device_name.clone()
                    }) {
                        Some(device) => Ok(device),
                        None => Err(AudioIoError::FailedToFindInputDevice),
                    }
                }
                Err(_) => Err(AudioIoError::FailedToGetInputDevice),
            },
            None => match self.host.default_input_device() {
                Some(device) => Ok(device),
                None => Err(AudioIoError::FailedToGetDefaultInput),
            },
        }
    }

    pub fn get_output_device(&self) -> Result<Device, AudioIoError> {
        match &self.output_device {
            Some(device_name) => match self.host.output_devices() {
                Ok(mut devices) => {
                    match devices.find(|d| {
                        d.name().unwrap_or(String::from("Unknown")) == device_name.clone()
                    }) {
                        Some(device) => Ok(device),
                        None => Err(AudioIoError::FailedToFindOutputDevice),
                    }
                }
                Err(_) => Err(AudioIoError::FailedToGetOutputDevice),
            },
            None => match self.host.default_output_device() {
                Some(device) => Ok(device),
                None => Err(AudioIoError::FailedToGetDefaultOutput),
            },
        }
    }

    pub fn set_input_device(&mut self, device_name: String) {
        self.input_device = Some(device_name);
    }

    pub fn set_output_device(&mut self, device_name: String) {
        self.output_device = Some(device_name);
    }
}
