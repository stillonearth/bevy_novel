use std::path::PathBuf;
use std::str::FromStr;

use bevy::prelude::*;
use renpy_parser::parsers::AST;

use bevy_kira_audio::prelude::*;

use crate::{
    find_element_with_index, list_ast_indices, MusicHandle, NovelBackground, NovelData, NovelImage,
    NovelSettings, NovelText, NovelTextWhat, NovelTextWho,
};

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

#[derive(Event)]
pub struct EventShowImageNode {}

#[derive(Event)]
pub struct EventHideImageNode {}

pub fn handle_play_audio(
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    music_handle: Res<MusicHandle>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    mut commands: Commands,
    mut er_play_audio: EventReader<EventPlayAudio>,
    plugin_settings: Res<NovelSettings>,
) {
    let base_path = PathBuf::from(&plugin_settings.assets_path);

    for event in er_play_audio.read() {
        let asset_path = base_path.join(event.filename.clone());
        let asset_handle = asset_server.load(asset_path);
        let mut play_event = audio.play(asset_handle);

        if event.audio_mode == AudioMode::Music {
            if let Some(handle) = music_handle.clone().0 {
                if let Some(instance) = audio_instances.get_mut(&handle) {
                    instance.stop(AudioTween::default());
                }
            }

            let handle = play_event.looped().handle();
            commands.insert_resource(MusicHandle(Some(handle)));
        }
    }
}

pub fn handle_start_scenario(
    mut er_start_scenario: EventReader<EventStartScenario>,
    mut novel_data: ResMut<NovelData>,
    mut ew_event_switch_next_node: EventWriter<EventSwitchNextNode>,
) {
    for event in er_start_scenario.read() {
        novel_data.current_index = 0;
        novel_data.ast = event.ast.clone();
        ew_event_switch_next_node.send(EventSwitchNextNode {});
    }
}

pub fn handle_switch_next_node(
    mut novel_data: ResMut<NovelData>,
    mut er_event_switch_next_node: EventReader<EventSwitchNextNode>,
    mut ew_handle_node: EventWriter<EventHandleNode>,
) {
    let mut switched = false;

    for _ in er_event_switch_next_node.read() {
        while !switched {
            let current_index = novel_data.current_index;

            let next_index = current_index + 1;
            let indices = list_ast_indices(novel_data.ast.clone());
            let max_index = *indices.iter().max().unwrap_or(&0);

            if next_index > max_index {
                switched = true;
                continue;
            }

            novel_data.current_index = next_index;

            let next_element = find_element_with_index(novel_data.ast.clone(), next_index);
            if next_element.is_some() {
                ew_handle_node.send(EventHandleNode {
                    ast: next_element.unwrap().clone(),
                });
                switched = true;
                continue;
            }

            for node in novel_data.ast.clone() {
                let next_element: Option<AST> = match node {
                    AST::Label(_, _, label_ast, _) => {
                        find_element_with_index(label_ast.clone(), next_index)
                    }
                    _ => None,
                };

                if next_element.is_some() && next_element.is_some() {
                    ew_handle_node.send(EventHandleNode {
                        ast: next_element.unwrap().clone(),
                    });
                    switched = true;
                    continue;
                }
            }
        }
    }
    // find element with required index
}

#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
pub fn handle_new_node(
    plugin_settings: Res<NovelSettings>,
    mut er_handle_node: EventReader<EventHandleNode>,
    mut ew_event_switch_next_node: EventWriter<EventSwitchNextNode>,
    mut ew_play_audio: EventWriter<EventPlayAudio>,
    mut ew_show_text_node: EventWriter<EventShowTextNode>,
    mut ew_hide_image_node: EventWriter<EventHideImageNode>,
    mut queries: ParamSet<(
        Query<(Entity, &mut Visibility, &mut Sprite, &mut NovelBackground)>,
        Query<(Entity, &mut Visibility, &mut Sprite, &mut NovelImage)>,
        Query<(Entity, &mut Visibility, &mut TextSpan, &NovelTextWhat)>,
        Query<(Entity, &mut Visibility, &mut TextSpan, &NovelTextWho)>,
    )>,
    assets: Res<AssetServer>,
    novel_data: Res<NovelData>,
) {
    let base_path = PathBuf::from(&plugin_settings.assets_path);

    for event in er_handle_node.read() {
        match event.ast.clone() {
            AST::Return(_, _) => {}
            AST::Jump(_, _, _) => {
                ew_event_switch_next_node.send(EventSwitchNextNode {});
            }
            AST::Scene(_, image, _layer) => {
                if let Some(img) = image {
                    for (_, mut v, mut sprite, _) in queries.p0().iter_mut() {
                        if let Some(spr) = novel_data.cached_images.get(&img) {
                            *sprite = spr.clone();
                        } else {
                            let image_name = format!("{}.png", img);
                            let image_path = base_path.join(image_name);
                            *sprite = Sprite::from_image(assets.load(image_path));
                        }
                        *v = Visibility::Visible;
                    }
                }

                ew_hide_image_node.send(EventHideImageNode {});
                ew_event_switch_next_node.send(EventSwitchNextNode {});
            }
            AST::Show(_, img) => {
                for (_, mut v, mut sprite, _) in queries.p1().iter_mut() {
                    if let Some(spr) = novel_data.cached_images.get(&img) {
                        *sprite = spr.clone();
                    } else {
                        let image_name = format!("{}.png", img);
                        let image_path = base_path.join(image_name);
                        *sprite = Sprite::from_image(assets.load(image_path));
                    }

                    *v = Visibility::Visible;
                }

                ew_event_switch_next_node.send(EventSwitchNextNode {});
            }
            AST::Hide(_, _) => {
                ew_hide_image_node.send(EventHideImageNode {});
                ew_event_switch_next_node.send(EventSwitchNextNode {});
            }
            AST::Label(_, _, _, _) => {
                ew_event_switch_next_node.send(EventSwitchNextNode {});
            }
            AST::GameMechanic(_, _) => {
                // ew_event_switch_next_node.send(EventSwitchNextNode {});
            }
            AST::LLMGenerate(_, _, _) => {
                // ew_event_switch_next_node.send(EventSwitchNextNode {});
            }
            AST::Play(_, mode, filename) => {
                let audio_mode = AudioMode::from_str(&mode).unwrap();

                ew_play_audio.send(EventPlayAudio {
                    filename,
                    audio_mode,
                });

                ew_event_switch_next_node.send(EventSwitchNextNode {});
            }
            AST::Say(_, who, what) => {
                for (_, _, mut text, _) in queries.p2().iter_mut() {
                    *text = TextSpan::new(what.clone());
                }

                for (_, _, mut text, _) in queries.p3().iter_mut() {
                    *text = TextSpan::new(who.clone().unwrap_or_default());
                }

                ew_show_text_node.send(EventShowTextNode {});
            }
            _ => {
                ew_event_switch_next_node.send(EventSwitchNextNode {});
            }
        }
    }
}

pub fn scale_images(
    mut commands: Commands,
    mut queries: ParamSet<(
        Query<(Entity, &mut Sprite, &mut NovelBackground)>,
        Query<(Entity, &mut Sprite, &mut Node, &mut NovelImage)>,
    )>,
    images: Res<Assets<Image>>,
    windows: Query<&Window>,
) {
    for (entity, sprite, mut node, _) in queries.p1().iter_mut() {
        // Manually scaling image width and height in proportion to window
        let image_handle = sprite.image.clone();

        if let Some(image) = images.get(&image_handle) {
            let sprite_height = image.height();

            let window = windows.get_single();
            if window.is_err() {
                return;
            }
            let window = window.unwrap();
            let window_height = window.height();

            let image_new_height = 0.75 * window_height;
            let image_scale = image_new_height as f32 / (sprite_height as f32);

            let image_transform = Transform::from_scale(Vec3::ONE * image_scale);

            node.margin.top = Val::Px(-(window_height - image_new_height) / 2.0);
            commands.entity(entity).insert(image_transform);
        }
    }

    for (entity, sprite, _) in queries.p0().iter_mut() {
        // Manually scaling image width and height in proportion to window
        let image_handle = sprite.image.clone();

        if let Some(image) = images.get(&image_handle) {
            let sprite_height = image.height();

            let window = windows.get_single();
            if window.is_err() {
                return;
            }
            let window = window.unwrap();
            let window_height = window.height();

            let image_new_height = 1.0 * window_height;
            let image_scale = image_new_height as f32 / (sprite_height as f32);

            let image_transform = Transform::from_scale(Vec3::ONE * image_scale);

            commands.entity(entity).insert(image_transform);
        }
    }
}

#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
pub fn handle_press_key(
    novel_settings: Res<NovelSettings>,
    keys: Res<ButtonInput<KeyCode>>,
    mut ew_switch_next_node: EventWriter<EventSwitchNextNode>,
) {
    if novel_settings.pause_handle_switch_node {
        return;
    }

    if keys.just_pressed(KeyCode::Space) {
        ew_switch_next_node.send(EventSwitchNextNode {});
    }
}

#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
pub fn handle_show_text_node(
    mut er_show_text_node: EventReader<EventShowTextNode>,
    mut paramset: ParamSet<(Query<(Entity, &mut Visibility, &NovelText)>,)>,
) {
    for _ in er_show_text_node.read() {
        for (_, mut visibility, _) in paramset.p0().iter_mut() {
            *visibility = Visibility::Visible;
        }
    }
}

#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
pub fn handle_hide_text_node(
    mut er_show_text_node: EventReader<EventHideTextNode>,
    mut paramset: ParamSet<(Query<(Entity, &mut Visibility, &NovelText)>,)>,
) {
    for _ in er_show_text_node.read() {
        for (_, mut visibility, _) in paramset.p0().iter_mut() {
            *visibility = Visibility::Hidden;
        }
    }
}

#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
pub fn handle_show_image_node(
    mut er_show_text_node: EventReader<EventShowImageNode>,
    mut paramset: ParamSet<(Query<(Entity, &mut Visibility, &NovelImage)>,)>,
) {
    for _ in er_show_text_node.read() {
        for (_, mut visibility, _) in paramset.p0().iter_mut() {
            *visibility = Visibility::Visible;
        }
    }
}

#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
pub fn handle_hide_image_node(
    mut er_show_text_node: EventReader<EventHideImageNode>,
    mut paramset: ParamSet<(Query<(Entity, &mut Visibility, &NovelImage)>,)>,
) {
    for _ in er_show_text_node.read() {
        for (_, mut visibility, _) in paramset.p0().iter_mut() {
            *visibility = Visibility::Hidden;
        }
    }
}
