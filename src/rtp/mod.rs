// For serialization prior to sending
use serde::{Deserialize, Serialize};

const RTP_HEADER_LENGTH: usize = 12;

#[derive(Serialize, Deserialize)]
pub struct Rtp {
    pub version: u8,
    pub padding: u8,
    pub extension: u8,
    pub csrc_count: u8,
    pub marker: u8,
    pub payload_type: u8,
    pub sequence: u16,
    pub timestamp: u32,
    pub ssrc: u32,
    pub csrc_list: Option<Vec<u32>>,
    pub payload: Vec<u8>,
}

impl Rtp {
    pub fn new() -> Rtp {
        Rtp {
            version: 2,
            padding: 0,
            extension: 0,
            csrc_count: 0,
            marker: 0,
            payload_type: 11,
            sequence: 0,
            timestamp: 0,
            ssrc: 55,
            csrc_list: None,
            payload: Vec::new(),
        }
    }

    pub fn serialize_to_vec_u8(self: Self) -> Vec<u8> {
        // The packed header
        #[derive(Serialize, Debug)]
        struct PackedRtp<'a> {
            byte_1: u8,
            byte_2: u8,
            seq_num: u16,
            timestamp: u32,
            ssrc: u32,
            payload: &'a Vec<u8>,
        }

        // Shift together the first byte
        let mut b1: u8 = (self.version << 6) | 0;
        b1 = (self.padding << 5) | b1;
        b1 = (self.extension << 4) | b1;
        b1 = (self.csrc_count) | b1;

        // Shift together the second byte
        let mut b2: u8 = (self.marker << 7) | 0;
        b2 = (self.payload_type) | b2;

        // Pack our packet
        let packed = PackedRtp {
            byte_1: b1,
            byte_2: b2,
            seq_num: u16::from_le(self.sequence),
            timestamp: u32::from_le(self.timestamp),
            ssrc: u32::from_le(self.ssrc),
            payload: &self.payload,
        };

        // Begin serialization
        let mut serialized: Vec<u8> = Vec::with_capacity(RTP_HEADER_LENGTH + &self.payload.len());

        serialized.push(packed.byte_1);
        serialized.push(packed.byte_2);
        serialized.extend_from_slice(&packed.seq_num.to_be_bytes());
        serialized.extend_from_slice(&packed.timestamp.to_be_bytes());
        serialized.extend_from_slice(&packed.ssrc.to_be_bytes());
        serialized.extend_from_slice(&packed.payload);

        serialized
    }

    pub fn get_audio_from_packet(packet: Vec<u8>) -> Vec<f32> {
        let audio = packet
            .chunks_exact(4)
            .map(|x| f32::from_le_bytes(x.try_into().unwrap()))
            .collect();

        audio
    }
}

pub struct RtpManager {
    pub sequence_num: u16,
    pub timestamp: u32,
}

impl RtpManager {
    pub fn new() -> RtpManager {
        RtpManager {
            sequence_num: 0,
            timestamp: 0,
        }
    }

    pub fn get_and_inc_vals(self: &mut Self) -> (u16, u32) {
        let vals = (self.sequence_num, self.timestamp);

        self.sequence_num = self.sequence_num + 1;
        self.timestamp = self.timestamp + 256;

        vals
    }
}

pub struct G711 {}

impl G711 {
    pub fn linear_to_ulaw(sample: &i16) -> u8 {
        let mut pcm_value = sample.to_owned();
        let sign = (pcm_value >> 8) & 0x80;

        if sign != 0 {
            pcm_value = -pcm_value;
        }

        if pcm_value > 32635 {
            pcm_value = 32635;
        }

        pcm_value += 0x84;
        let mut exponent: i16 = 7;
        let mut mask = 0x4000;

        while pcm_value & mask == 0 {
            exponent -= 1;
            mask >>= 1;
        }

        let manitssa: i16 = (pcm_value >> (exponent + 3)) & 0x0f;
        let ulaw_value = sign | exponent << 4 | manitssa;
        (!ulaw_value) as u8
    }
}
