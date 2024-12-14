use bevy::{input::common_conditions::input_toggle_active, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_novel::{events::EventStartScenario, NovelPlugin};
use renpy_parser::parse_scenario;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(
            WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::Escape)),
        )
        .add_plugins(NovelPlugin {})
        .add_systems(Startup, (setup_camera, start_visual_novel))
        .run();
}

fn start_visual_novel(mut ew_start_scenario: EventWriter<EventStartScenario>) {
    let path = "assets/script.rpy";
    let result = parse_scenario(path);

    if result.is_err() {
        panic!("{:?}", result.err());
    }

    let (ast, _) = result.unwrap();
    ew_start_scenario.send(EventStartScenario { ast });
}

fn setup_camera(mut cmd: Commands) {
    cmd.spawn(Camera2d {});
}
