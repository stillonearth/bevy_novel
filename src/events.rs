use std::str::FromStr;

use bevy::prelude::*;
use renpy_parser::parsers::AST;

use crate::rpy_asset_loader::Blob;

#[derive(Clone, Event)]
pub struct Scene {
    pub image: String,
}

#[derive(Clone, Event)]
pub struct EventShow {
    pub image: String,
}

#[derive(Clone, Event)]
pub struct EventJump {
    pub label: String,
}

#[derive(Clone, Event)]
pub struct EventLabel {
    pub label: String,
}

#[derive(Clone, Event)]
pub struct EventReturn {}

#[derive(Clone, Event)]
pub struct EventLoadScenario {
    pub filename: String,
}

#[derive(Clone, Event)]
pub struct EventScenarioLoaded {
    pub blob_handle: Handle<Blob>,
    pub filename: String,
}

#[derive(Clone, Event)]
pub struct EventStartScenario {
    pub ast: Vec<AST>,
}

#[derive(Clone, Event)]
pub struct EventSwitchNextNode {}

#[derive(Clone, Event)]
pub struct EventHandleNode {
    pub ast: AST,
}

#[derive(Clone, Event)]
pub struct EventSay {
    pub data: String,
}

#[derive(Clone, PartialEq)]
pub enum AudioMode {
    Sound,
    Music,
    Voice,
}

// implement from_string trait for audio mode
impl FromStr for AudioMode {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let empty_string = "";
        let character = "\"";
        let s = s.replace(character, empty_string);

        match s.as_ref() {
            "sound" => Ok(AudioMode::Sound),
            "music" => Ok(AudioMode::Music),
            "voice" => Ok(AudioMode::Voice),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Event)]
pub struct EventPlayAudio {
    pub filename: String,
    pub audio_mode: AudioMode,
}

#[derive(Event)]
pub struct EventShowTextNode {}

#[derive(Event)]
pub struct EventHideTextNode {}
