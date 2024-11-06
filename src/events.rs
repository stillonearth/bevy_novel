use bevy::prelude::*;
use renpy_parser::parsers::AST;

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
