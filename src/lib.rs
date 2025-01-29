pub mod events;
pub mod rpy_asset_loader;

use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

use renpy_parser::parsers::{inject_node, AST};

use events::*;

#[derive(Component)]
struct NovelBackground {}

#[derive(Component)]
struct NovelImage;

#[derive(Component)]
struct NovelTextWhat;

#[derive(Component)]
struct NovelTextWho;
pub struct NovelPlugin;

#[derive(Resource, Clone)]
struct MusicHandle(Option<Handle<AudioInstance>>);

#[derive(Resource, Default)]
pub struct NovelData {
    pub ast: Vec<AST>,
    current_index: usize,
    pub cached_images: HashMap<String, Sprite>,
}

impl NovelData {
    pub fn push_text_node(&mut self, who: Option<String>, what: String, index: usize) {
        let node = AST::Say(index, who, what);
        self.ast = inject_node(self.ast.clone(), node.clone());
        for a in self.ast.iter_mut() {
            if let AST::Label(node_index, label, node_ast, opts) = a {
                let first_index = node_ast.first().unwrap().index();
                let last_index = node_ast.last().unwrap().index();

                if index >= first_index && index <= last_index {
                    *a = AST::Label(
                        *node_index,
                        label.clone(),
                        inject_node(node_ast.clone(), node.clone()).clone(),
                        opts.clone(),
                    )
                }
            }
        }
    }

    pub fn write_image_cache(&mut self, image_name: String, sprite: Sprite) {
        println!("writing cache image: {}", image_name);
        self.cached_images.insert(image_name, sprite);
    }

    pub fn push_scene_node(&mut self, image: String, index: usize) {
        let node = AST::Scene(index, Some(image), "master".into());
        self.ast = inject_node(self.ast.clone(), node.clone());
        for a in self.ast.iter_mut() {
            if let AST::Label(node_index, label, node_ast, opts) = a {
                let first_index = node_ast.first().unwrap().index();
                let last_index = node_ast.last().unwrap().index();

                if index >= first_index && index <= last_index {
                    *a = AST::Label(
                        *node_index,
                        label.clone(),
                        inject_node(node_ast.clone(), node.clone()).clone(),
                        opts.clone(),
                    )
                }
            }
        }
    }
}

#[derive(Resource, Default)]
pub struct NovelSettings {
    pub assets_path: String,
    pub pause_handle_switch_node: bool,
}

impl Plugin for NovelPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AudioPlugin)
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    handle_start_scenario,
                    handle_switch_next_node,
                    handle_new_node,
                    handle_press_key,
                    handle_play_audio,
                    handle_show_text_node,
                    handle_hide_text_node,
                ),
            )
            .add_event::<EventHandleNode>()
            .add_event::<EventHideTextNode>()
            .add_event::<EventJump>()
            .add_event::<EventLabel>()
            .add_event::<EventPlayAudio>()
            .add_event::<EventReturn>()
            .add_event::<EventSay>()
            .add_event::<EventShow>()
            .add_event::<EventShowTextNode>()
            .add_event::<EventStartScenario>()
            .add_event::<EventSwitchNextNode>()
            .init_resource::<NovelData>()
            .insert_resource(MusicHandle(None))
            .insert_resource(NovelSettings::default())
            .init_asset_loader::<rpy_asset_loader::RpyAssetLoader>()
            .init_asset::<rpy_asset_loader::Rpy>();
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Name::new("Character Image"),
        Sprite { ..default() },
        NovelImage {},
        ZIndex(1),
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        Visibility::Hidden,
    ));
    commands.spawn((
        Name::new("Background Image"),
        Sprite { ..default() },
        NovelBackground {},
        ZIndex(2),
        Node {
            position_type: PositionType::Absolute,
            width: Val::Auto,
            height: Val::Auto,
            ..default()
        },
        Visibility::Hidden,
    ));

    commands
        .spawn((
            Text::default(),
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(20.0),
                left: Val::Px(20.0),
                ..default()
            },
        ))
        .with_children(|p| {
            p.spawn((
                TextSpan::new(""),
                NovelTextWho {},
                Name::new("Text Who"),
                TextLayout::new_with_justify(JustifyText::Left),
                Visibility::Visible,
            ));

            p.spawn((
                TextSpan::new("\n"),
                Name::new("Text Span"),
                TextLayout::new_with_justify(JustifyText::Left),
            ));

            p.spawn((
                TextSpan::new(""),
                NovelTextWhat {},
                Name::new("Text What"),
                TextLayout::new_with_justify(JustifyText::Left),
                Visibility::Visible,
            ));
        });
}

fn handle_play_audio(
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

fn handle_start_scenario(
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

fn find_element_with_index(ast: Vec<AST>, index: usize) -> Option<AST> {
    for ast in ast.iter() {
        let ast_index = match ast {
            AST::Return(i, _) => *i,
            AST::Jump(i, _, _) => *i,
            AST::Scene(i, _, _) => *i,
            AST::Show(i, _) => *i,
            AST::Hide(i, _) => *i,
            AST::Label(i, _, _, _) => *i,
            AST::Init(i, _, _) => *i,
            AST::Say(i, _, _) => *i,
            AST::Play(i, _, _) => *i,
            AST::Define(i, _) => *i,
            AST::Stop(i, _, _, _) => *i,
            AST::GameMechanic(i, _) => *i,
            AST::LLMGenerate(i, _, _) => *i,
            AST::Error => panic!(),
        };

        if index == ast_index {
            return Some(ast.clone());
        }
    }

    None
}

fn list_ast_indices(ast: Vec<AST>) -> Vec<usize> {
    let mut indices: Vec<usize> = ast
        .iter()
        .map(|a| match a {
            AST::Return(i, _) => *i,
            AST::Jump(i, _, _) => *i,
            AST::Scene(i, _, _) => *i,
            AST::Show(i, _) => *i,
            AST::Hide(i, _) => *i,
            AST::Label(i, _, _, _) => *i,
            AST::Init(i, _, _) => *i,
            AST::Say(i, _, _) => *i,
            AST::Play(i, _, _) => *i,
            AST::Define(i, _) => *i,
            AST::Stop(i, _, _, _) => *i,
            AST::GameMechanic(i, _) => *i,
            AST::LLMGenerate(i, _, _) => *i,
            AST::Error => panic!(),
        })
        .collect();

    for ast in ast {
        match ast {
            AST::Label(_, _, vec, _) => {
                indices.extend_from_slice(list_ast_indices(vec).as_slice());
            }
            AST::Init(_, vec, _) => {
                indices.extend_from_slice(list_ast_indices(vec).as_slice());
            }
            _ => {}
        }
    }

    indices
}

fn handle_switch_next_node(
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
fn handle_new_node(
    plugin_settings: Res<NovelSettings>,
    mut er_handle_node: EventReader<EventHandleNode>,
    mut ew_event_switch_next_node: EventWriter<EventSwitchNextNode>,
    mut ew_play_audio: EventWriter<EventPlayAudio>,
    mut ew_show_text_node: EventWriter<EventShowTextNode>,
    mut ew_hide_text_node: EventWriter<EventHideTextNode>,
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
                // insert images

                if let Some(img) = image {
                    for (_, mut v, mut sprite, _) in queries.p0().iter_mut() {
                        println!("getting image: {}", img);

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

                for (_, mut visibility, _, _) in queries.p1().iter_mut() {
                    *visibility = Visibility::Hidden;
                }

                ew_event_switch_next_node.send(EventSwitchNextNode {});
            }
            AST::Show(_, img) => {
                for (_, mut v, mut sprite, _) in queries.p1().iter_mut() {
                    // todo introduce delay for image load
                    let image_name = format!("{}.png", img);
                    let image_path = base_path.join(image_name);
                    *sprite = Sprite::from_image(assets.load(image_path));
                    *v = Visibility::Visible;
                }

                ew_event_switch_next_node.send(EventSwitchNextNode {});
            }
            AST::Hide(_, _) => {
                ew_event_switch_next_node.send(EventSwitchNextNode {});
            }
            AST::Label(_, _, _, _) => {
                ew_event_switch_next_node.send(EventSwitchNextNode {});
            }
            AST::Init(_, _, _) => {
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

                ew_hide_text_node.send(EventHideTextNode {});
                ew_show_text_node.send(EventShowTextNode {});
            }
            _ => {
                ew_event_switch_next_node.send(EventSwitchNextNode {});
            }
        }
    }
}

#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
fn handle_press_key(
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
fn handle_show_text_node(
    mut er_show_text_node: EventReader<EventShowTextNode>,
    mut paramset: ParamSet<(
        Query<(Entity, &mut Visibility, &mut Text, &NovelTextWhat)>,
        Query<(Entity, &mut Visibility, &mut Text, &NovelTextWho)>,
    )>,
) {
    for _ in er_show_text_node.read() {
        for (_, mut visibility, _, _) in paramset.p0().iter_mut() {
            *visibility = Visibility::Visible;
        }

        for (_, mut visibility, _, _) in paramset.p1().iter_mut() {
            *visibility = Visibility::Visible;
        }
    }
}

#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
fn handle_hide_text_node(
    mut er_show_text_node: EventReader<EventHideTextNode>,
    mut paramset: ParamSet<(
        Query<(Entity, &mut Visibility, &mut Text, &NovelTextWhat)>,
        Query<(Entity, &mut Visibility, &mut Text, &NovelTextWho)>,
    )>,
) {
    for _ in er_show_text_node.read() {
        for (_, mut visibility, _, _) in paramset.p0().iter_mut() {
            *visibility = Visibility::Hidden;
        }

        for (_, mut visibility, _, _) in paramset.p1().iter_mut() {
            *visibility = Visibility::Hidden;
        }
    }
}
