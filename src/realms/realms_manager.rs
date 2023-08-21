use crate::realms::realm::{ChannelType, Realm};
use crate::realms::realm_desc::RealmDescription;
use crate::types::{ChannelIdSize, NumRealmsSize, RealmIdSize, UserIdSize};
use crate::user::User;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

impl Clone for RealmsManager {
    fn clone(&self) -> RealmsManager {
        RealmsManager {
            realms: self.realms.clone(),
            num_realms: self.num_realms,
        }
    }
}

#[derive(Default, Serialize, Deserialize, PartialEq, Debug)]
pub struct RealmsManager {
    // Map holding all of the server realms
    realms: HashMap<RealmIdSize, Realm>,

    // Right now this is only used for generating an ID.
    // Not necessary once a database is in play
    num_realms: NumRealmsSize,
}

impl RealmsManager {
    pub fn add_realm(&mut self, realm_name: String) -> RealmIdSize {
        let id = self.generate_realm_id();
        self.add_realm_with_id(id, realm_name);

        id
    }

    pub fn add_realm_with_id(&mut self, realm_id: RealmIdSize, realm_name: String) {
        self.realms
            .insert(realm_id, Realm::new(realm_id, realm_name));
    }

    pub fn add_channel(&mut self, realm_id: RealmIdSize, channel_type: ChannelType, name: String) {
        if let Some(realm) = self.realms.get_mut(&realm_id) {
            realm.add_channel(channel_type, name);
        }
    }

    pub fn add_user(&mut self, realm_id: RealmIdSize, user: User) {
        if let Some(realm) = self.realms.get_mut(&realm_id) {
            realm.add_user(user.get_id(), user);
        }
    }

    pub fn remove_user(&mut self, realm_id: RealmIdSize, user_id: UserIdSize) {
        if let Some(realm) = self.realms.get_mut(&realm_id) {
            realm.remove_user(user_id);
        }
    }

    // Super lazy generation of realm id
    pub fn generate_realm_id(&mut self) -> RealmIdSize {
        let id = self.num_realms;
        self.num_realms += 1;
        id
    }

    pub fn get_realm_descriptions(&self) -> Vec<RealmDescription> {
        let mut realm_descriptions = Vec::new();
        for realm in self.realms.values() {
            realm_descriptions.push(RealmDescription::new(
                *realm.get_id(),
                realm.get_name().clone(),
                &realm.text_channels,
                &realm.voice_channels,
            ));
        }

        realm_descriptions
    }

    pub fn get_realms(&self) -> Vec<(&RealmIdSize, &String)> {
        let mut realms = Vec::new();
        for realm in self.realms.values() {
            realms.push((realm.get_id(), realm.get_name()));
        }
        realms
    }

    pub fn get_realm(&self, realm_id: RealmIdSize) -> Option<&Realm> {
        self.realms.get(&realm_id)
    }

    pub fn get_realm_mut(&mut self, realm_id: RealmIdSize) -> Option<&mut Realm> {
        self.realms.get_mut(&realm_id)
    }

    pub fn clear(&mut self) {
        self.realms.clear();
        self.num_realms = 0;
    }

    pub fn add_user_to_voice_channel(
        &mut self,
        user_id: UserIdSize,
        realm_id: RealmIdSize,
        channel_id: ChannelIdSize,
    ) {
        if let Some(realm) = self.realms.get_mut(&realm_id) {
            for channel in realm.get_voice_channels_mut() {
                if channel.0 == &channel_id {
                    // Don't add the same user more than once
                    if !channel.1.connected_users.contains(&user_id) {
                        channel.1.connected_users.push(user_id);
                    }
                }
            }
        }
    }

    pub fn remove_user_from_voice_channel(
        &mut self,
        user_id: UserIdSize,
        realm_id: RealmIdSize,
        channel_id: ChannelIdSize,
    ) {
        if let Some(realm) = self.realms.get_mut(&realm_id) {
            for channel in realm.get_voice_channels_mut() {
                if channel.0 == &channel_id {
                    if let Some(index) =
                        channel.1.connected_users.iter().position(|x| x == &user_id)
                    {
                        channel.1.connected_users.remove(index);
                    }
                }
            }
        }
    }
}
