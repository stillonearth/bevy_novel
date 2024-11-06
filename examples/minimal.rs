use bevy::{input::common_conditions::input_toggle_active, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_lunex::{prelude::MainUi, UiMinimalPlugins};
use bevy_rl::{events::EventStartScenario, NovelPlugin};
use renpy_parser::{
    group_logical_lines,
    lexer::Lexer,
    list_logical_lines,
    parsers::{parse_block, AST},
};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, UiMinimalPlugins))
        .add_plugins(
            WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::Escape)),
        )
        .add_plugins(NovelPlugin {})
        .add_systems(Startup, (setup_camera, start_visual_novel))
        .run();
}

fn load_scenario(path: String) -> Vec<AST> {
    let lines = list_logical_lines(&path).unwrap();
    let blocks = group_logical_lines(lines).unwrap();

    let l = &mut Lexer::new(blocks.clone(), true);

    let (ast, _) = parse_block(l);
    return ast;
}

fn start_visual_novel(mut ew_start_scenario: EventWriter<EventStartScenario>) {
    let path = "assets/script.rpy";
    let ast = load_scenario(path.to_string());

    ew_start_scenario.send(EventStartScenario { ast });
}

fn setup_camera(mut cmd: Commands) {
    cmd.spawn((
        MainUi,
        Camera2dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 1000.0),
            camera: Camera::default(),
            ..default()
        },
    ));
}
