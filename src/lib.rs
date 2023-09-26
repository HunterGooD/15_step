mod combat;
mod enemy;
mod entities;
mod events;
mod loading;
mod map;
mod player;
mod ui;

use crate::enemy::EnemyPlugin;
use crate::loading::LoadingPlugin;
use crate::map::MapPlugin;
use crate::player::PlayerPlugin;
use crate::ui::menu::MenuPlugin;

use bevy::app::App;

#[cfg(debug_assertions)]
use bevy::diagnostic::{
    FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin, SystemInformationDiagnosticsPlugin,
};
use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    #[default]
    Loading,
    InGame,
    Menu,
}

#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
enum InGameState {
    Pause,
    #[default]
    Play,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GameState>()
            .add_state::<InGameState>()
            .add_plugins((
                LoadingPlugin,
                MenuPlugin,
                MapPlugin,
                PlayerPlugin,
                EnemyPlugin,
            ));

        #[cfg(debug_assertions)]
        {
            app.add_plugins((
                FrameTimeDiagnosticsPlugin,
                LogDiagnosticsPlugin::default(),
                WorldInspectorPlugin::default(),
                SystemInformationDiagnosticsPlugin::default(),
            ));
        }
    }
}
