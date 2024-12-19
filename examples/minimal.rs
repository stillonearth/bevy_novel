use bevy::{input::common_conditions::input_toggle_active, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_novel::{events::EventStartScenario, rpy_asset_loader::Blob, NovelPlugin};
use renpy_parser::{group_logical_lines, lexer::Lexer, parse_logical_lines, parsers::parse_block};

#[derive(Resource)]
struct Scenario {
    pub content: Handle<Blob>,
    pub is_loaded: bool,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(
            WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::Escape)),
        )
        .add_plugins(NovelPlugin {})
        .add_systems(Startup, (setup_camera, load_scenario))
        .add_systems(Update, (start_visual_novel,))
        .init_asset::<bevy_novel::rpy_asset_loader::Blob>()
        .run();
}

fn load_scenario(mut commands: Commands, asset_server: Res<AssetServer>) {
    let blob_handle: Handle<Blob> = asset_server.load("script.rpy");
    commands.insert_resource(Scenario {
        content: blob_handle,
        is_loaded: false,
    });
}

fn start_visual_novel(
    mut ew_start_scenario: EventWriter<EventStartScenario>,
    blob_assets: Res<Assets<Blob>>,
    mut scenario: ResMut<Scenario>,
) {
    if scenario.is_loaded {
        return;
    }

    if let Some(blob) = blob_assets.get(scenario.content.id()) {
        let content = std::str::from_utf8(&blob.bytes).unwrap().to_string();

        let lines = parse_logical_lines(content, "filename.rpy".to_string()).unwrap();

        let blocks = group_logical_lines(lines).unwrap();
        let l = &mut Lexer::new(blocks.clone(), true);

        let (ast, _) = parse_block(l);

        ew_start_scenario.send(EventStartScenario { ast });
        scenario.is_loaded = true;
    }
}

fn setup_camera(mut cmd: Commands) {
    cmd.spawn(Camera2d {});
}
