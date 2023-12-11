use crate::realms::channels::text_channel::TextChannel;
use crate::realms::channels::voice_channel::VoiceChannel;
use crate::types::{ChannelIdSize, RealmIdSize};
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
        text_channels_map: &HashMap<ChannelIdSize, TextChannel>,
        voice_channels_map: &HashMap<ChannelIdSize, VoiceChannel>,
    ) -> RealmDescription {
        let mut text_channels = Vec::new();
        let mut voice_channels = Vec::new();

        for tc in text_channels_map.values() {
            text_channels.push((*tc.get_id(), tc.get_name().clone()))
        }

        for vc in voice_channels_map.values() {
            voice_channels.push((*vc.get_id(), vc.get_name().clone()))
        }

        RealmDescription {
            id,
            name,
            text_channels,
            voice_channels,
        }
    }

    pub fn get_text_channels(&self) -> Vec<(ChannelIdSize, String)> {
        self.text_channels.clone()
    }

    pub fn get_voice_channels(&self) -> Vec<(ChannelIdSize, String)> {
        self.voice_channels.clone()
    }
}
