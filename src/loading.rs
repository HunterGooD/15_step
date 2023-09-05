use crate::GameState;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(GameState::Loading).continue_to_state(GameState::Menu),
        )
        .add_collection_to_loading_state::<_, TextureAssets>(GameState::Loading)
        .add_collection_to_loading_state::<_, PlayerTexture>(GameState::Loading);
    }
}

#[derive(AssetCollection, Resource)]
pub struct TextureAssets {
    #[asset(path = "tiles/forest/tileset.png")]
    // #[asset(path = "/mnt/programming/rust/15_step_game_beginer/assets/tiles/forest/tileset.png")]
    pub tile: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub struct PlayerTexture {
    #[asset(path = "player/adventurer.png")]
    pub sprite: Handle<Image>,
}

// #[derive(AssetCollection, Resource)]
// pub struct SpritePlayer {
//     #[asset(path="")]
//     pub sprite_player: Handle<Image>
// }
