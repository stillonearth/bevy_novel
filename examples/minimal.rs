use bevy::{input::common_conditions::input_toggle_active, prelude::*};
use bevy_defer::AsyncPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_novel::{events::EventStartScenario, rpy_asset_loader::Rpy, NovelPlugin};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            AsyncPlugin::default_settings(),
            WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::Escape)),
        ))
        .add_plugins(NovelPlugin {})
        .init_state::<AppState>()
        .add_systems(Startup, (setup_camera, load_scenario))
        .add_systems(
            Update,
            start_visual_novel.run_if(in_state(AppState::Loading)),
        )
        .run();
}

fn load_scenario(mut commands: Commands, asset_server: Res<AssetServer>) {
    let scenario_handle = ScenarioHandle(asset_server.load("script.rpy"));
    println!("loading");
    commands.insert_resource(scenario_handle);
}

fn setup_camera(mut cmd: Commands) {
    cmd.spawn(Camera2d {});
}

fn start_visual_novel(
    mut ew_start_scenario: EventWriter<EventStartScenario>,
    scenario: Res<ScenarioHandle>,
    rpy_assets: Res<Assets<Rpy>>,
    mut state: ResMut<NextState<AppState>>,
) {
    if let Some(rpy) = rpy_assets.get(scenario.id()) {
        println!("im here");
        ew_start_scenario.send(EventStartScenario { ast: rpy.0.clone() });
        state.set(AppState::Novel);
    }
}

#[derive(Resource, Deref, DerefMut)]
struct ScenarioHandle(Handle<Rpy>);

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    #[default]
    Loading,
    Novel,
}
