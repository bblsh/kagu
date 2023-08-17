use crate::text_channel::TextChannel;
use crate::types::{ChannelIdSize, RealmIdSize};
use crate::voice_channel::VoiceChannel;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;

// A RealmDescription is a description of the realm
// Instead of sending everything like chat history, active members, etc
// we send a high-level, smaller set of data for each realm to users.
// Additional information may be requested and sent in other messages.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct RealmDescription {
    pub id: RealmIdSize,
    pub name: String,
    pub text_channels: Vec<(ChannelIdSize, String)>,
    pub voice_channels: Vec<(ChannelIdSize, String)>,
}

impl RealmDescription {
    pub fn new(
        id: RealmIdSize,
        name: String,
        text_channels: &HashMap<ChannelIdSize, TextChannel>,
        voice_channels: &HashMap<ChannelIdSize, VoiceChannel>,
    ) -> RealmDescription {
        let mut t_channels = Vec::new();
        let mut v_channels = Vec::new();

        for (_id, tc) in text_channels {
            t_channels.push((tc.get_id().clone(), tc.get_name().clone()))
        }

        for (_id, vc) in voice_channels {
            v_channels.push((vc.get_id().clone(), vc.get_name().clone()))
        }

        RealmDescription {
            id: id,
            name: name,
            text_channels: t_channels,
            voice_channels: v_channels,
        }
    }

    pub fn get_text_channels(&self) -> Vec<(ChannelIdSize, String)> {
        self.text_channels.clone()
    }

    pub fn get_voice_channels(&self) -> Vec<(ChannelIdSize, String)> {
        self.voice_channels.clone()
    }
}
