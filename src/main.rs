use std::fs::File;
use std::io::Write;
use std::path::Path;

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    tasks::IoTaskPool,
    utils::Duration,
    window::{PresentMode, WindowMode},
};
use bevy_ecs_tilemap::prelude::*;

use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::prelude::*;
use leafwing_input_manager::InputManagerBundle;

fn main() {
    // create App main struct for run game in bevy
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "It's a game!".into(),
                        resolution: (1920., 1080.).into(),
                        resizable: false,
                        present_mode: PresentMode::AutoVsync,
                        // mode: WindowMode::Fullscreen,
                        mode: WindowMode::Windowed,
                        ..default()
                    }),
                    ..default()
                }),
        )
        // for save scene
        .register_type::<Name>()
        .register_type::<Player>()
        .register_type::<JumpInfo>()
        .register_type::<DashInfo>()
        .register_type::<PlayerActions>()
        .register_type::<PlayerStates>()
        .register_type::<bevy::math::Rect>()
        .register_type::<core::option::Option<bevy::math::Rect>>()
        .register_type::<core::option::Option<bevy::ecs::entity::Entity>>()
        .register_type::<core::option::Option<bevy::math::f32::Vec2>>()
        // end register type
        // register events
        .add_event::<StopJump>()
        .add_event::<SlideEvent>()
        // end register events
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(bevy::diagnostic::SystemInformationDiagnosticsPlugin::default())
        .add_plugin(bevy::diagnostic::EntityCountDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin)
        .add_plugin(WorldInspectorPlugin::default())
        .add_plugin(TilemapPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0)) // create world rapier physics
        .insert_resource(RapierConfiguration {
            gravity: Vec2::new(0.0, -2000.0),
            ..default()
        })
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(InputManagerPlugin::<PlayerActions>::default()) // player actions for buttons
        .add_plugin(InputManagerPlugin::<CameraActions>::default())
        .add_startup_system(setup_graphics)
        .add_startup_system(setup_player.run_if(save_file_not_exist))
        .add_startup_system(spawn_enemy)
        .add_startup_system(setup_map.run_if(save_file_not_exist))
        .add_startup_system(load_scene.run_if(not(save_file_not_exist)))
        .add_system(camera_settings)
        .add_system(move_player)
        .add_system(follow)
        .add_system(player_collision)
        // .add_system(save_scene) // TODO: reflect bevy_rapier_2d collider to serialize scene
        .run();
}

fn follow(
    time: Res<Time>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
    player_query: Query<&Transform, With<Player>>,
) {
    for mut cam_t in &mut camera_query {
        if let Ok(player_t) = player_query.get_single() {
            let velocity = Vec2::new(cam_t.translation.x, cam_t.translation.y).lerp(
                Vec2::new(player_t.translation.x, player_t.translation.y + 150.),
                20. * time.delta_seconds(),
            );

            cam_t.translation = velocity.extend(cam_t.translation.z);
        }
    }
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
enum PlayerActions {
    Move,
    Jump,
    Dash,
    Save,
}

// for debug
#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
enum CameraActions {
    Zoom,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Default, Reflect)]
enum PlayerStates {
    #[default]
    Idle,
    Run,
    Jump,
    Fall,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Default, Reflect)]
enum EnemyStates {
    #[default]
    Idle,
    Run,
    Atack,
}

#[derive(Clone, Default, Debug, Component, Reflect)]
#[reflect(Component)]
struct Name(String);

#[derive(Clone, Default, Debug, Component, Reflect)]
#[reflect(Component)]
struct Player(Name);

#[derive(Clone, Default, Debug, Component, Reflect)]
#[reflect(Component)]
struct Enemy(Name);

#[derive(Clone, Default, Debug, Component, Reflect)]
#[reflect(Component)]
struct ActiveEntity<T: Default> {
    pub rotation: i8,
    pub velocity: Vec2,
    pub current_state: T,
}

#[derive(Clone, Default, Debug, Component, Reflect)]
#[reflect(Component)]
struct Ground(Name);

const MAX_JUMP: u8 = 2;

#[derive(Resource, Debug, Reflect)]
#[reflect(Resource)]
struct JumpInfo {
    count: u8,
    time_up: Timer,
}

impl Default for JumpInfo {
    fn default() -> Self {
        Self {
            count: 0,
            time_up: Timer::new(Duration::from_millis(500), TimerMode::Once),
        }
    }
}

#[derive(Default, Debug)]
struct StopJump;

#[derive(Default, Debug)]
struct SlideEvent(WallPosition);

#[derive(Default, Debug, Clone)]
enum WallPosition {
    #[default]
    Right,
    Left,
}

const DASH_FRAMES: u8 = 7;
const DASH_SPEED_FACTOR: f32 = 0.1;

#[derive(Resource, Debug, Reflect)]
#[reflect(Resource)]
struct DashInfo {
    cooldown_time: Timer,
    evade_time: Timer,
    is_used: bool,
    frames_count: u8,
}

impl Default for DashInfo {
    fn default() -> Self {
        let mut cooldown_timer = Timer::new(Duration::from_millis(350), TimerMode::Once);
        cooldown_timer.tick(Duration::from_secs_f32(1.5)); // finished on init
        Self {
            cooldown_time: cooldown_timer,
            evade_time: Timer::new(Duration::from_millis(50), TimerMode::Once),
            is_used: false,
            frames_count: 0,
        }
    }
}

#[derive(Bundle, Default)]
struct PlayerBundle {
    name: Player,
    player: ActiveEntity<PlayerStates>,
    sprite: SpriteBundle,
    input: InputManagerBundle<PlayerActions>,
    rigid_body: RigidBody,
    controller: KinematicCharacterController,
    controller_output: KinematicCharacterControllerOutput,
    collider: Collider,
}

//TODO: create abstract entity bundle

#[derive(Bundle, Default)]
struct EnemyBundle {
    name: Enemy,
    enemy: ActiveEntity<EnemyStates>,
    sprite: SpriteBundle,
    rigid_body: RigidBody,
    controller: KinematicCharacterController,
    controller_output: KinematicCharacterControllerOutput,
    collider: Collider,
}

fn setup_graphics(mut commands: Commands) {
    // Add a camera so we can see the debug-render.
    commands
        .spawn(Camera2dBundle::default())
        .insert(InputManagerBundle::<CameraActions> {
            input_map: InputMap::default()
                .insert(SingleAxis::mouse_wheel_y(), CameraActions::Zoom)
                .build(),
            ..default()
        });
}

fn camera_settings(
    mut q: Query<(&mut OrthographicProjection, &ActionState<CameraActions>), With<Camera2d>>,
) {
    const CAMERA_ZOOM_RATE: f32 = 0.05;

    let (mut projection, action_state) = q.single_mut();
    let zoom_delta = action_state.value(CameraActions::Zoom);

    projection.scale *= 1. - zoom_delta * CAMERA_ZOOM_RATE;
}

fn setup_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    let texture_handle: Handle<Image> = asset_server.load("tiles/forest/tileset.png"); // 21 x 15 tiles
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
        texture: TilemapTexture::Single(texture_handle.clone()),
        tile_size,
        transform: Transform::from_xyz(0., tile_y, 1.).with_scale(Vec3::splat(tile_scale)),
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

fn spawn_enemy(mut commands: Commands) {
    commands
        .spawn(EnemyBundle {
            name: Enemy(Name("Slime".to_string())),
            enemy: ActiveEntity {
                rotation: 1,
                velocity: Vec2::default(),
                current_state: EnemyStates::default(),
            },
            sprite: SpriteBundle {
                sprite: Sprite {
                    color: Color::hex("fc0303").unwrap(),
                    custom_size: Some(Vec2::splat(30f32)),
                    ..default()
                },
                transform: Transform::from_xyz(200f32, 100f32, 10.),
                ..default()
            },
            rigid_body: RigidBody::KinematicVelocityBased,
            controller: KinematicCharacterController {
                slide: true,
                // filter_groups: Some(CollisionGroups::new(
                //     Group::from_bits(0b1001).unwrap(),
                //     Group::from_bits(0b1101).unwrap(),
                // )),
                ..default()
            },
            controller_output: KinematicCharacterControllerOutput::default(),
            collider: Collider::cuboid(15., 15.),
        })
        .insert(Sensor);
}

fn setup_player(mut commands: Commands) {
    commands.spawn(PlayerBundle {
        name: Player(Name("Player".to_string())),
        player: ActiveEntity {
            rotation: 1,
            velocity: Vec2::default(),
            current_state: PlayerStates::default(),
        },
        sprite: SpriteBundle {
            sprite: Sprite {
                color: Color::hex("00fc43").unwrap(),
                custom_size: Some(Vec2::new(60., 140.)),
                ..Default::default()
            },
            transform: Transform::from_xyz(-100., 400., 10.),
            ..Default::default()
        },
        input: InputManagerBundle::<PlayerActions> {
            action_state: ActionState::default(),
            input_map: InputMap::default()
                .insert(KeyCode::Escape, PlayerActions::Save) // delete this
                .insert(DualAxis::left_stick(), PlayerActions::Move)
                .insert(VirtualDPad::wasd(), PlayerActions::Move)
                .insert(VirtualDPad::arrow_keys(), PlayerActions::Move)
                .insert(KeyCode::Space, PlayerActions::Jump)
                .insert(GamepadButtonType::South, PlayerActions::Jump)
                .insert(KeyCode::LShift, PlayerActions::Dash)
                .insert(GamepadButtonType::East, PlayerActions::Dash)
                .set_gamepad(Gamepad { id: 0 })
                .build(),
        },
        rigid_body: RigidBody::KinematicVelocityBased,
        controller: KinematicCharacterController {
            slide: true,
            autostep: None,
            // filter_groups: Some(CollisionGroups::new(
            //     Group::from_bits(0b1101).unwrap(),
            //     Group::from_bits(0b1001).unwrap(),
            // )),
            ..default()
        },
        controller_output: KinematicCharacterControllerOutput::default(),
        collider: Collider::cuboid(30., 70.),
    });
}

const SCENE_FILE_PATH: &str = "data/scenes/test_scene.scn.ron";

fn save_file_not_exist() -> bool {
    let path_file = format!("assets/{SCENE_FILE_PATH}");
    let path_file = Path::new(&path_file);
    !path_file.exists()
}

fn load_scene(mut cmd: Commands, asset_server: Res<AssetServer>) {
    cmd.spawn(DynamicSceneBundle {
        scene: asset_server.load(SCENE_FILE_PATH),
        ..default()
    });
}

fn save_scene(world: &mut World) {
    let mut q = world.query::<&ActionState<PlayerActions>>();
    let actions = q.single(world);
    let save_pressed = actions.just_pressed(PlayerActions::Save);

    if !save_pressed {
        return;
    }
    // let scene = World::new();
    info!("Save start!");
    let mut player = world.query_filtered::<Entity, With<Player>>();
    // TODO: change collider to ground component and create startup function generate colliders, set input player
    let mut colliders = world.query_filtered::<Entity, With<Collider>>();
    // let mut map = world.query_filtered::<Entity, With<TilemapSize>>();
    let mut scene = DynamicSceneBuilder::from_world(world);
    scene.extract_entities(player.iter(world));
    scene.extract_entities(colliders.iter(world));
    // scene.extract_entities(map.iter(world));

    let type_registry = world.resource::<AppTypeRegistry>();
    let ron_scene = match scene.build().serialize_ron(&type_registry) {
        Ok(s) => s,
        Err(err) => {
            info!("{:?}", err);
            return;
        }
    }; // dialog window error
    #[cfg(not(target_arch = "wasm32"))]
    IoTaskPool::get()
        .spawn(async move {
            // Write the scene RON data to file
            File::create(format!("assets/{SCENE_FILE_PATH}"))
                .and_then(|mut file| file.write(ron_scene.as_bytes()))
                .expect("Error while writing scene to file");
        })
        .detach();
    info!("Save end!");
}

fn move_player(
    time: Res<Time>,
    rapier_config: Res<RapierConfiguration>,
    mut jump_info: Local<JumpInfo>,
    mut dash_info: Local<DashInfo>,
    mut controller_query: Query<
        (
            &ActionState<PlayerActions>,
            &mut ActiveEntity<PlayerStates>,
            &mut KinematicCharacterController,
            Option<&KinematicCharacterControllerOutput>,
        ),
        With<Player>,
    >,
    mut stop_jump: EventReader<StopJump>,
    mut sliding: EventReader<SlideEvent>,
) {
    let (action_state, mut player, mut controller, controller_output) =
        controller_query.single_mut();

    let grounded = match controller_output {
        Some(out) => out.grounded,
        None => false,
    };

    let dt = time.delta_seconds();

    // TODO: global variable
    let speed = 58.0;
    let jump_impulse = 1000.0;
    let mut instant_acceleration = Vec2::ZERO;
    let mut instant_velocity = player.velocity;

    let axis_vector = action_state
        .clamped_axis_pair(PlayerActions::Move)
        .unwrap()
        .x();
    if axis_vector != 0. {
        player.rotation = if (axis_vector * 9.) < 0. { -1 } else { 1 };
    }

    // physics simulation
    if grounded {
        // friction
        instant_velocity.x *= 0.9;
    } else {
        // friction in jump
        instant_velocity.x *= 0.95;
        // gravity
        if !grounded {
            instant_acceleration += Vec2::Y * rapier_config.gravity;

            if !sliding.is_empty() {
                let mut pos_slide = WallPosition::default();
                for e in sliding.iter() {
                    pos_slide = e.0.clone();
                }
                let pos_slide = match pos_slide {
                    WallPosition::Left => -1.,
                    WallPosition::Right => 1.,
                };
                if pos_slide == axis_vector {
                    instant_velocity.y = 1.; // reset inertia
                    jump_info.count = 1; // TODO: implement method for count jumps
                    info!("|||||||||||||||||||||||SLIDING|||||||||||||||||||||||");
                    let slide_scale = -21.;
                    instant_acceleration += Vec2::Y * slide_scale;
                    sliding.clear();
                }
            }
        }
    }
    let mut y = 0.;
    for action in action_state.get_just_pressed() {
        match action {
            PlayerActions::Jump => {
                if jump_info.count >= MAX_JUMP {
                    return;
                }
                // instant_acceleration.y = 1.;
                instant_velocity.y = 1.;
                y = jump_impulse;
                jump_info.time_up.reset();

                jump_info.count += 1;
                player.current_state = PlayerStates::Jump;

                dash_info.is_used = false;
            }
            PlayerActions::Dash => {
                if dash_info.cooldown_time.finished() {
                    dash_info.evade_time.reset();
                    dash_info.frames_count = 0;
                    dash_info.is_used = true;
                }
            }
            _ => (),
        }
    }

    // info!("{:?}", dash_info);

    if !dash_info.cooldown_time.finished() {
        dash_info.cooldown_time.tick(time.delta());
    }
    if !stop_jump.is_empty() {
        stop_jump.clear();
        jump_info.time_up.tick(Duration::from_secs(1));
        instant_velocity.y = 1.;
    }
    if player.current_state == PlayerStates::Jump {
        jump_info.time_up.tick(time.delta());
        if jump_info.time_up.finished() {
            // if player.velocity.y <= 0. {
            player.current_state = PlayerStates::Fall;
        }
    } else {
        if grounded {
            jump_info.count = 0;
            if axis_vector == 0. {
                player.current_state = PlayerStates::Idle;
            } else {
                player.current_state = PlayerStates::Run;
            }
        } else {
            player.current_state = PlayerStates::Fall;
        }
    }

    if dash_info.is_used {
        dash_info.evade_time.tick(time.delta());
        instant_velocity.x *= DASH_SPEED_FACTOR;
        dash_info.frames_count += 1;
        if dash_info.frames_count == DASH_FRAMES {
            dash_info.is_used = false;
            dash_info.cooldown_time.reset();
        }
        controller.translation = Some(Vec2::new(player.rotation as f32 * 35., 0.));
    }

    instant_velocity += Vec2::new(axis_vector * speed, y);
    instant_velocity = instant_velocity.clamp(Vec2::splat(-1000.0), Vec2::splat(1000.0));
    player.velocity = (instant_acceleration * dt) + instant_velocity;
    let translation = controller.translation.unwrap_or(Vec2::new(0., 0.)) + player.velocity * dt;
    controller.translation = Some(translation);
}

fn player_collision(
    mut cmd: Commands,
    q: Query<(&KinematicCharacterControllerOutput, &Transform), With<Player>>,
    mut stop_jump: EventWriter<StopJump>,
    mut slide: EventWriter<SlideEvent>,
) {
    // info!("Start collision detect");
    for (out, transform) in q.iter() {
        let y_top_player = transform.translation.y + 68.;
        let y_down_player = transform.translation.y - 68.;
        // info!("Player transform {:?}", transform);
        for collision in &out.collisions {
            if y_top_player < collision.toi.witness1.y {
                // up collision player
                // so that the hero starts falling when he hits the ceiling
                stop_jump.send_default();
            }

            if y_down_player < collision.toi.witness1.y && y_top_player > collision.toi.witness1.y {
                // left and right collision
                // slide wall
                if collision.toi.witness1.x > transform.translation.x {
                    slide.send(SlideEvent(WallPosition::Right));
                } else {
                    slide.send(SlideEvent(WallPosition::Left));
                }
            }

            if y_down_player > collision.toi.witness1.y {
                // ground
            }

            // cmd.spawn(SpriteBundle{
            //     sprite: Sprite { color: Color::hex("34c6eb").unwrap(), custom_size: Some(Vec2::new(3f32, 10f32)), ..default() }, transform: Transform::from_translation(collision.toi.witness1.extend(0.)), ..default()
            // });
            // cmd.spawn(SpriteBundle{
            //     sprite: Sprite { color: Color::hex("000094").unwrap(), custom_size: Some(Vec2::new(3f32, 10f32)), ..default() }, transform: Transform::from_translation(collision.toi.witness2.extend(0.)), ..default()
            // });

            // cmd.spawn(SpriteBundle{
            //     sprite: Sprite { color: Color::hex("34c6eb").unwrap(), custom_size: Some(Vec2::new(10f32, 3f32)), ..default() }, transform: Transform::from_translation(collision.toi.witness1.extend(0.)), ..default()
            // });
            // cmd.spawn(SpriteBundle{
            //     sprite: Sprite { color: Color::hex("000094").unwrap(), custom_size: Some(Vec2::new(10f32, 3f32)), ..default() }, transform: Transform::from_translation(collision.toi.witness2.extend(0.)), ..default()
            // });

            // info!("{:?}", collision);
        }
    }
    // info!("End collision detect");
}
// 2023-07-24T13:17:44.160618Z  INFO first_game: CharacterCollision { entity: 53v0, character_translation: Vec2(749.32275, 82.03036), character_rotation: 0.0, translation_applied: Vec2(0.0, 0.0), translation_remaining: Vec2(16.93062, -17.50391), toi: Toi { toi: 0.009614948, witness1: Vec2(780.0, 145.2335), witness2: Vec2(29.999996, 63.894253), normal1: Vec2(-1.0, 3.520538e-5), normal2: Vec2(1.0, -3.520538e-5), status: Converged } }
//2023-07-24T13:19:55.576577Z  INFO first_game: CharacterCollision { entity: 82v0, character_translation: Vec2(-99.98577, -268.6), character_rotation: 0.0, translation_applied: Vec2(0.0, 0.0), translation_remaining: Vec2(0.0, -16.575462), toi: Toi { toi: 0.013999939, witness1: Vec2(-90.80467, -340.0), witness2: Vec2(9.181595, -70.00001), normal1: Vec2(-0.0, 1.0), normal2: Vec2(0.0, -1.0), status: Converged } }
//2023-07-24T13:21:03.825734Z  INFO first_game: CharacterCollision { entity: 82v0, character_translation: Vec2(-1636.5319, -268.9542), character_rotation: 0.0, translation_applied: Vec2(-9.517902, 0.0), translation_remaining: Vec2(0.0, -16.410189), toi: Toi { toi: 0.010458237, witness1: Vec2(-1624.2828, -340.0), witness2: Vec2(12.24823, -70.00001), normal1: Vec2(-0.0007295108, 0.99999976), normal2: Vec2(0.0007295108, -0.99999976), status: Converged } }
