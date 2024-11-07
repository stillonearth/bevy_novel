pub mod events;

use std::str::FromStr;

use bevy::{prelude::*, sprite::Anchor};
use bevy_lunex::prelude::*;
use events::*;
use renpy_parser::parsers::AST;

#[derive(Debug, Component)]
struct NovelBackground {}

#[derive(Debug, Component)]
struct NovelImage;

#[derive(Debug, Component)]
struct NovelText;

pub struct NovelPlugin;

// simple derive, to set all fields to their defaults
#[derive(Resource, Default)]
struct NovelData {
    ast: Vec<AST>,
    current_index: usize,
}

impl Plugin for NovelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    handle_start_scenario,
                    handle_switch_next_node,
                    handle_new_node,
                    handle_press_key,
                ),
            )
            .add_event::<EventShow>()
            .add_event::<EventJump>()
            .add_event::<EventLabel>()
            .add_event::<EventReturn>()
            .add_event::<EventSay>()
            .add_event::<EventStartScenario>()
            .add_event::<EventSwitchNextNode>()
            .add_event::<EventHandleNode>()
            .add_event::<EventPlayAudio>()
            .init_resource::<NovelData>();
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
                NovelText {},
            ));

            ui.spawn((
                UiLink::<MainUi>::path("Root/Rectangle/Image"),
                UiLayout::solid().pack::<Base>(),
                UiImage2dBundle::from(assets.load("character igor.png")),
                NovelImage {},
            ))
            .insert(Visibility::Hidden);
        });
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
            AST::Say(i, _, _, _) => *i,
            AST::UserStatement(i, _) => *i,
            AST::Play(i, _, _) => *i,
            AST::Error => {
                todo!()
            }
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
            AST::Say(i, _, _, _) => *i,
            AST::UserStatement(i, _) => *i,
            AST::Play(i, _, _) => *i,
            AST::Error => {
                todo!()
            }
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
    mut q_images: ParamSet<(
        Query<(Entity, &mut Visibility, &mut NovelBackground)>,
        Query<(Entity, &mut Visibility, &mut NovelImage)>,
    )>,
    mut q_text: Query<(Entity, &mut Text, &NovelText)>,
    assets: Res<AssetServer>,
) {
    for event in er_handle_node.read() {
        println!("{:?}", event.ast);

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
                    for (entity, mut v, _) in q_images.p0().iter_mut() {
                        let image_name = format!("{}.png", img);
                        let image: Handle<Image> = assets.load(image_name);
                        commands.entity(entity).insert(image);
                        *v = Visibility::Visible;
                    }
                }

                ew_event_switch_next_node.send(EventSwitchNextNode {});
            }
            AST::Show(_, img) => {
                for (entity, mut v, _) in q_images.p1().iter_mut() {
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
            }
            AST::UserStatement(_, value) => {
                // for now onlyu first person speech
                if value.as_bytes()[0] as char != "\"".chars().next().unwrap() {
                    continue;
                }

                for (_, mut text, _) in q_text.iter_mut() {
                    *text = Text::from_section(
                        value.clone(),
                        TextStyle {
                            font: assets.load("font.ttf"),
                            font_size: 30.0,
                            color: Color::WHITE,
                        },
                    );
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
        println!("handle_press_key");
        ew_switch_next_node.send(EventSwitchNextNode {});
    }
}
