use crate::loading::TextureAssets;
use crate::GameState;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_rapier2d::prelude::*;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_plugins(TilemapPlugin)
            .add_systems(OnEnter(GameState::InGame), setup_map);
    }
}

fn setup_map(mut commands: Commands, asset: Res<TextureAssets>) {
    let texture_handle = asset.tile.clone(); // dont load tiles in bevy_ecs_tilemap
    let map_size = TilemapSize { x: 21, y: 15 };

    let mut tile_storage = TileStorage::empty(map_size);
    let tilemap_entity = commands.spawn_empty().id();

    let mut map = vec![
        [
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
        [
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
        [
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
        [
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
        [
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
        [
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
        [
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
        [
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
        [
            -1, -1, -1, -1, -1, -1, -1, 0, 1, 1, 1, 1, 1, 1, 3, -1, -1, -1, -1, -1, -1,
        ],
        [
            -1, -1, -1, -1, -1, -1, -1, 63, 64, 65, 64, 65, 64, 65, 66, -1, -1, -1, -1, -1, -1,
        ],
        [
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
        [
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
        [
            -1, -1, -1, 1, -1, -1, -1, -1, -1, 1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
        [
            -1, -1, 1, 1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
        [
            0, 1, 2, 1, 2, 1, 2, 1, 2, 1, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 3,
        ],
    ];
    map.reverse();

    let mut pos_x = 0f32;
    let mut pos_y = -400f32;
    let tile_scale = 5f32;
    let colider_block = 24f32 * tile_scale;
    for y in 0..map_size.y {
        for x in 0..map_size.x {
            let tile_idx = map[y as usize][x as usize]; // FIXME: not safe operation
            if tile_idx == -1 {
                pos_x += colider_block;
                continue;
            }
            let tile_pos = TilePos { x, y };
            let idx = TileTextureIndex(tile_idx as u32);
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    texture_index: idx,
                    ..Default::default()
                })
                .id();
            // Here we let the tile storage component know what tiles we have.
            tile_storage.set(&tile_pos, tile_entity);

            commands
                .spawn(Collider::cuboid(colider_block / 2f32, colider_block / 2f32))
                .insert(RigidBody::Fixed)
                .insert(TransformBundle::from_transform(Transform::from_xyz(
                    pos_x, pos_y, 0.,
                )));
            pos_x += colider_block;
        }
        pos_y += colider_block;
        pos_x = 0f32;
    }

    let tile_size = TilemapTileSize { x: 24.0, y: 24.0 };
    let grid_size = TilemapGridSize { x: 24., y: 24. };
    let map_type = TilemapType::default();

    let tile_y = -400.;

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        tile_size,
        transform: Transform::from_xyz(0., tile_y, 10.).with_scale(Vec3::splat(tile_scale)),
        ..Default::default()
    });

    let colider_radius = 200.;
    commands
        .spawn(Collider::cuboid(15000., colider_radius))
        .insert(RigidBody::Fixed)
        .insert(TransformBundle::from_transform(Transform::from_xyz(
            4500.,
            tile_y + (tile_size.y * tile_scale) / 2. - colider_radius,
            0.,
        )));
    commands
        .spawn(Collider::cuboid(10., 200.))
        .insert(RigidBody::Fixed)
        .insert(TransformBundle::from_transform(Transform::from_xyz(
            -200., 200., 0.,
        )));
}
