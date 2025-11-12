pub mod messages;
pub mod rpy_asset_loader;

use std::collections::HashMap;

use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

use renpy_parser::parsers::{AST, inject_node};

use messages::*;

#[derive(Component)]
pub struct NovelBackground;

#[derive(Component)]
pub struct NovelImage;

#[derive(Component)]
pub struct NovelText;

#[derive(Component)]
pub struct NovelTextWhat;

#[derive(Component)]
pub struct NovelTextWho;
pub struct NovelPlugin;

#[derive(Resource, Clone)]
pub struct MusicHandle(Option<Handle<AudioInstance>>);

#[derive(Resource, Default)]
pub struct NovelData {
    pub ast: Vec<AST>,
    pub current_index: usize,
    pub cached_images: HashMap<String, Sprite>,
}

impl NovelData {
    // Manipulate Scenario

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

    pub fn push_show_node(&mut self, image: String, index: usize) {
        let node = AST::Show(index, image);
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

    pub fn push_hide_node(&mut self, image: String, index: usize) {
        let node = AST::Hide(index, image);
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

    // Manipulate Images

    pub fn write_image_cache(&mut self, image_name: String, sprite: Sprite) {
        self.cached_images.insert(image_name, sprite);
    }
}

#[derive(Resource, Default)]
pub struct NovelSettings {
    pub assets_path: String,
    pub pause_handle_switch_node: bool,
}

impl Plugin for NovelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    adjust_text_position,
                    handle_start_scenario,
                    handle_switch_next_node,
                    handle_new_node,
                    (handle_hide_image_node, handle_hide_text_node).chain(),
                    (handle_show_image_node, handle_show_text_node).chain(),
                    handle_press_key,
                    handle_play_audio,
                    scale_images,
                )
                    .chain(),
            )
            .add_message::<EventHandleNode>()
            .add_message::<EventHideImageNode>()
            .add_message::<EventHideTextNode>()
            .add_message::<EventJump>()
            .add_message::<EventLabel>()
            .add_message::<EventPlayAudio>()
            .add_message::<EventReturn>()
            .add_message::<EventSay>()
            .add_message::<EventShow>()
            .add_message::<EventShowImageNode>()
            .add_message::<EventShowTextNode>()
            .add_message::<EventStartScenario>()
            .add_message::<EventSwitchNextNode>()
            .add_message::<EventNovelEnd>()
            .init_resource::<NovelData>()
            .insert_resource(MusicHandle(None))
            .insert_resource(NovelSettings::default())
            .init_asset_loader::<rpy_asset_loader::RpyAssetLoader>()
            .init_asset::<rpy_asset_loader::Rpy>();
    }
}

fn setup(mut commands: Commands, _novel_settings: Res<NovelSettings>) {
    commands
        .spawn((
            Text2d::default(),
            NovelText,
            Name::new("Novel Text"),
            ZIndex(10),
            Transform::from_translation(Vec3::Z),
            TextLayout::new(Justify::Left, LineBreak::WordBoundary),
        ))
        .with_children(|p| {
            p.spawn((
                TextSpan::new(""),
                NovelTextWho {},
                Name::new("Novel Text Who"),
                Visibility::Visible,
            ));

            p.spawn((TextSpan::new("\n"), Name::new("Novel Text Newline")));

            p.spawn((
                TextSpan::new(""),
                NovelTextWhat {},
                Name::new("Novel Text What"),
                Transform::from_translation(Vec3::Z),
                Visibility::Visible,
            ));
        });

    commands.spawn((
        Name::new("Background Image"),
        Sprite::default(),
        NovelBackground,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Auto,
            height: Val::Auto,
            ..default()
        },
        Visibility::Hidden,
    ));

    commands.spawn((
        Name::new("Character Image"),
        Sprite::default(),
        NovelImage,
        // ZIndex(2),
        Node {
            position_type: PositionType::Absolute,
            width: Val::Auto,
            height: Val::Auto,
            ..default()
        },
        Visibility::Hidden,
    ));
}

fn adjust_text_position(
    mut windows: Query<&Window>,
    mut q_novel_text: Query<(Entity, &mut Transform, &NovelText)>,
) {
    let window = windows.single_mut().unwrap(); // Get window

    for (_, mut transform, _) in q_novel_text.iter_mut() {
        transform.translation.y = -window.resolution.height() / 2.0 + 50.0;
        transform.translation.x = -window.resolution.width() / 3.0 + 50.0;
    }
}

pub fn find_element_with_index(ast: Vec<AST>, index: usize) -> Option<AST> {
    for ast in ast.iter() {
        let ast_index = match ast {
            AST::Return(i, _) => *i,
            AST::Jump(i, _, _) => *i,
            AST::Scene(i, _, _) => *i,
            AST::Show(i, _) => *i,
            AST::Hide(i, _) => *i,
            AST::Label(i, _, _, _) => *i,
            AST::Say(i, _, _) => *i,
            AST::Play(i, _, _) => *i,
            AST::Define(i, _) => *i,
            AST::Stop(i, _, _, _) => *i,
            AST::GameMechanic(i, _) => *i,
            AST::LLMGenerate(i, _, _) => *i,
            AST::Comment(i, _) => *i,
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
            AST::Say(i, _, _) => *i,
            AST::Play(i, _, _) => *i,
            AST::Define(i, _) => *i,
            AST::Stop(i, _, _, _) => *i,
            AST::GameMechanic(i, _) => *i,
            AST::LLMGenerate(i, _, _) => *i,
            AST::Comment(i, _) => *i,
            AST::Error => panic!(),
        })
        .collect();

    for ast in ast {
        match ast {
            AST::Label(_, _, vec, _) => {
                indices.extend_from_slice(list_ast_indices(vec).as_slice());
            }
            _ => {}
        }
    }

    indices
}
