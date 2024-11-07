use bevy::{prelude::*, sprite::Anchor};
use bevy_kira_audio::prelude::*;
use bevy_lunex::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, UiMinimalPlugins))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut cmd: Commands, assets: Res<AssetServer>) {
    // Spawn camera
    cmd.spawn((
        MainUi,
        Camera2dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 1000.0),
            camera: Camera::default(),
            ..default()
        },
    ));

    // Spawn UiTree
    cmd.spawn((
        UiTreeBundle::<MainUi> {
            tree: UiTree::new2d("MyUiSystem"),
            ..default()
        },
        SourceFromCamera,
    ))
    .with_children(|ui| {
        // Spawn boundary node
        ui.spawn((
            // Link this widget
            UiLink::<MainUi>::path("Text"),
            // Here we can define where we want to position our text within the parent node,
            // don't worry about size, that is picked up and overwritten automaticaly by Lunex to match text size.
            UiLayout::window()
                .pos(Rl((5., 80.)))
                .anchor(Anchor::CenterLeft)
                .pack::<Base>(),
            // Add text
            UiText2dBundle {
                text: Text::from_section(
                    "Hello world! Hello world! Hello world! Hello world! Hello world! Hello world!",
                    TextStyle::default(),
                ),
                ..default()
            },
            UiTextSize::new().size(Rh(5.0)),
        ));
    });
}
