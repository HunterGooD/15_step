use bevy::{
    prelude::*,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    window::{PresentMode, WindowMode}
};
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::prelude::*;
use leafwing_input_manager::InputManagerBundle;

fn main() {
    // create App main struct for run game in bevy 
    App::new()
    .add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "It's a game!".into(),
            resolution: (1920., 1080.).into(),
            resizable: false,
            present_mode: PresentMode::AutoVsync,
            mode: WindowMode::Fullscreen,
            ..default()
        }),
        ..default()
    })) 
    .add_plugin(LogDiagnosticsPlugin::default())
    .add_plugin(FrameTimeDiagnosticsPlugin)
    .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0)) // create world rapier physics
    .insert_resource(RapierConfiguration {
        gravity: Vec2::new(0.0, -2000.0),
        ..default()
    }) 
    .add_plugin(RapierDebugRenderPlugin::default())
    .add_plugin(InputManagerPlugin::<PlayerActions>::default()) // player actions for buttons 
    .add_system(setup_graphics)
    .run();
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
enum PlayerActions {
    Move, 
    Jump 
}

#[derive(Clone, Default, Debug, Component)]
struct Name(String);

#[derive(Clone, Default, Debug, Component)]
struct Player;

fn setup_graphics(mut commands: Commands) {
    // Add a camera so we can see the debug-render.
    commands.spawn(Camera2dBundle::default());
}