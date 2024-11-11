pub mod events;

use std::str::FromStr;

use bevy::{prelude::*, sprite::Anchor};
use bevy_kira_audio::prelude::*;
use bevy_lunex::prelude::*;
use events::*;
use renpy_parser::parsers::AST;

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

// simple derive, to set all fields to their defaults
#[derive(Resource, Default)]
struct NovelData {
    ast: Vec<AST>,
    current_index: usize,
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
                ),
            )
            .add_event::<EventShow>()
            .add_event::<EventJump>()
            .add_event::<EventLabel>()
            .add_event::<EventReturn>()
            .add_event::<EventSay>()
            .add_event::<EventStartScenario>()
            .add_event::<EventSwitchNextNode>()
            .add_event::<EventShowTextNode>()
            .add_event::<EventHandleNode>()
            .add_event::<EventPlayAudio>()
            .init_resource::<NovelData>()
            .insert_resource(MusicHandle(None));
    }
}

fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    commands
        .spawn((
            UiTreeBundle::<MainUi> {
                tree: UiTree::new2d("NovelUISystem"),
                ..default()
            },
            SourceFromCamera,
        ))
        .with_children(|ui| {
            ui.spawn((
                UiLink::<MainUi>::path("Root"),
                UiLayout::boundary()
                    .pos1(Ab(10.0))
                    .pos2(Rl(100.0) - Ab(10.0))
                    .pack::<Base>(),
            ));

            ui.spawn((
                UiLink::<MainUi>::path("Root/Rectangle"),
                UiLayout::solid()
                    .size((Rl(100.0), Rl(100.0)))
                    .pack::<Base>(),
                UiImage2dBundle::from(assets.load("inverted.png")),
                NovelBackground {},
            ))
            .insert(Visibility::Hidden);

            ui.spawn((
                UiLink::<MainUi>::path("Root/Rectangle/Text"),
                UiLayout::window()
                    .pos(Rl((5., 80.)))
                    .anchor(Anchor::CenterLeft)
                    .pack::<Base>(),
                UiText2dBundle {
                    text: Text::from_section(
                        "",
                        TextStyle {
                            font: assets.load("font.ttf"),
                            font_size: 30.0,
                            color: Color::WHITE,
                        },
                    ),
                    ..default()
                },
                UiTextSize::new().size(Rh(5.0)),
                NovelTextWhat {},
            ));

            ui.spawn((
                UiLink::<MainUi>::path("Root/Rectangle/Text"),
                UiLayout::window()
                    .pos(Rl((5., 80.)))
                    .anchor(Anchor::CenterLeft)
                    .pack::<Base>(),
                UiText2dBundle {
                    text: Text::from_section(
                        "who",
                        TextStyle {
                            font: assets.load("font.ttf"),
                            font_size: 30.0,
                            color: Color::WHITE,
                        },
                    ),
                    ..default()
                },
                UiTextSize::new().size(Rh(5.0)),
                NovelTextWho {},
            ));

            ui.spawn((
                UiLink::<MainUi>::path("Root/Rectangle/Image"),
                UiLayout::solid().pack::<Base>(),
                UiImage2dBundle::from(assets.load("character komarito.png")),
                NovelImage {},
            ))
            .insert(Visibility::Hidden);
        });
}

fn handle_play_audio(
    mut commands: Commands,
    mut er_play_audio: EventReader<EventPlayAudio>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    music_handle: Res<MusicHandle>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
) {
    for event in er_play_audio.read() {
        let mut play_event = audio.play(asset_server.load(event.filename.clone()));

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
            AST::UserStatement(i, _) => *i,
            AST::Play(i, _, _) => *i,
            AST::Error => {
                todo!()
            }
            AST::Define(i, _) => *i,
            AST::Stop(i, _, _, _) => *i,
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
            AST::UserStatement(i, _) => *i,
            AST::Play(i, _, _) => *i,
            AST::Error => {
                todo!()
            }
            AST::Define(i, _) => *i,
            AST::Stop(i, _, _, _) => *i,
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

fn handle_new_node(
    mut commands: Commands,
    mut er_handle_node: EventReader<EventHandleNode>,
    mut ew_event_switch_next_node: EventWriter<EventSwitchNextNode>,
    mut ew_play_audio: EventWriter<EventPlayAudio>,
    mut ew_show_text_node: EventWriter<EventShowTextNode>,
    mut queries: ParamSet<(
        Query<(Entity, &mut Visibility, &mut NovelBackground)>,
        Query<(Entity, &mut Visibility, &mut NovelImage)>,
        Query<(Entity, &mut Visibility, &mut Text, &NovelTextWhat)>,
    )>,
    assets: Res<AssetServer>,
) {
    for event in er_handle_node.read() {
        match event.ast.clone() {
            AST::Return(_, _) => {
                println!("Over");
            }
            AST::Jump(_, _, _) => {
                ew_event_switch_next_node.send(EventSwitchNextNode {});
            }
            AST::Scene(_, image, _layer) => {
                // insert images

                if let Some(img) = image {
                    for (entity, mut v, _) in queries.p0().iter_mut() {
                        let image_name = format!("{}.png", img);
                        let image: Handle<Image> = assets.load(image_name);
                        commands.entity(entity).insert(image);
                        *v = Visibility::Visible;
                    }
                }

                for (_, mut visibility, _) in queries.p1().iter_mut() {
                    *visibility = Visibility::Hidden;
                }

                ew_event_switch_next_node.send(EventSwitchNextNode {});
            }
            AST::Show(_, img) => {
                for (entity, mut v, _) in queries.p1().iter_mut() {
                    let image_name = format!("{}.png", img);
                    let image: Handle<Image> = assets.load(image_name);
                    commands.entity(entity).insert(image);
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
            AST::Play(_, mode, filename) => {
                let audio_mode = AudioMode::from_str(&mode).unwrap();

                ew_play_audio.send(EventPlayAudio {
                    filename,
                    audio_mode,
                });

                ew_event_switch_next_node.send(EventSwitchNextNode {});
            }
            AST::Say(_, who, what) => {
                for (_, mut visibility, mut text, _) in queries.p2().iter_mut() {
                    *text = Text::from_section(
                        what.clone(),
                        TextStyle {
                            font: assets.load("font.ttf"),
                            font_size: 30.0,
                            color: Color::WHITE,
                        },
                    );

                    // Changing text without hiding it causes jumpy rendering
                    // Change text, hide it and show the next frame works better
                    *visibility = Visibility::Hidden;

                    ew_show_text_node.send(EventShowTextNode {});
                }
            }
            _ => {
                println!("handle unknown");
            }
        }
    }
}

fn handle_press_key(
    keys: Res<ButtonInput<KeyCode>>,
    mut ew_switch_next_node: EventWriter<EventSwitchNextNode>,
) {
    if keys.just_pressed(KeyCode::Space) {
        ew_switch_next_node.send(EventSwitchNextNode {});
    }
}

fn handle_show_text_node(
    mut er_show_text_node: EventReader<EventShowTextNode>,
    mut q_text: Query<(Entity, &mut Visibility, &mut Text, &NovelTextWhat)>,
) {
    for _ in er_show_text_node.read() {
        for (_, mut visibility, _, _) in q_text.iter_mut() {
            *visibility = Visibility::Visible;
        }
    }
}

// ---------------
// Internal Events
// ---------------

#[derive(Event)]
struct EventShowTextNode {}
