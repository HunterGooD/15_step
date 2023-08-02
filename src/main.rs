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
        .register_type::<AttackCollider>()
        .register_type::<Health>()
        .register_type::<Enemy>()
        .register_type::<Goblin>()
        .register_type::<Slime>()
        // end register type
        // register events
        .add_event::<StopJump>()
        .add_event::<SlideEvent>()
        // end register events
        .add_plugins((
            LogDiagnosticsPlugin::default(),
            bevy::diagnostic::SystemInformationDiagnosticsPlugin::default(),
            bevy::diagnostic::EntityCountDiagnosticsPlugin::default(),
            FrameTimeDiagnosticsPlugin,
            WorldInspectorPlugin::default(),
            TilemapPlugin,
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0), // create world rapier physics
            RapierDebugRenderPlugin::default(),
            InputManagerPlugin::<PlayerActions>::default(), // player actions for buttons
            InputManagerPlugin::<CameraActions>::default(),
        ))
        .insert_resource(RapierConfiguration {
            gravity: Vec2::new(0.0, -2000.0),
            ..default()
        })
        .add_systems(
            Startup,
            (
                setup_graphics,
                setup_player.run_if(save_file_not_exist),
                spawn_enemies,
                setup_map.run_if(save_file_not_exist),
                load_scene.run_if(not(save_file_not_exist)),
            ),
        )
        .add_systems(
            Update,
            (
                camera_settings,
                move_player,
                move_enemy_slime,
                move_enemy_goblin,
                follow,
                player_attack,
                attack_checker.run_if(isset_collider),
                player_collision,
                sensor_event,
            ),
        )
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
    Attack,
}

// for debug
#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
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

// **********************************************************  STATS
// add speed how stats
// stats for weapon > attack damage, attack speed, attack range, attack interrupt, attack knockback
// Strength * attack damage = true damage
#[derive(Clone, Debug, Component, Reflect)]
#[reflect(Component)]
struct Health(i32);

impl Default for Health {
    fn default() -> Self {
        Self(100)
    }
}

#[derive(Clone, Default, Debug, Component, Reflect)]
#[reflect(Component)]
struct Armor(i32);

#[derive(Clone, Default, Debug, Component, Reflect)]
#[reflect(Component)]
struct Strength(i32);

// *********************************************************  END STATS

#[derive(Clone, Default, Debug, Component, Reflect)]
#[reflect(Component)]
struct AttackCollider(Option<Entity>);

#[derive(Clone, Default, Debug, Component, Reflect)]
#[reflect(Component)]
struct Ground(Name); // for ground collider name

const MAX_JUMP: u8 = 2;

#[derive(Resource, Debug, Reflect)]
#[reflect(Resource)]
struct TimeAttack(Timer);

impl Default for TimeAttack {
    fn default() -> Self {
        Self(Timer::new(Duration::from_millis(380), TimerMode::Once))
    }
}

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

#[derive(Event, Default, Debug)]
struct StopJump;

#[derive(Event, Default, Debug)]
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
struct Stats {
    health: Health,
    armor: Armor,
    strength: Strength,
}

#[derive(Bundle, Default)]
struct PlayerBundle {
    name: Player,
    player: ActiveEntity<PlayerStates>,
    stats: Stats,
    sprite: SpriteBundle,
    input: InputManagerBundle<PlayerActions>,
    rigid_body: RigidBody,
    controller: KinematicCharacterController,
    controller_output: KinematicCharacterControllerOutput,
    collider: Collider,
    attack: AttackCollider,
}

//TODO: create abstract entity bundle

#[derive(Bundle, Default)]
struct EnemyBundle<T: Default + Component> {
    name: Enemy,
    monster_type: T,
    stats: Stats,
    enemy: ActiveEntity<EnemyStates>,
    sprite: SpriteBundle,
    rigid_body: RigidBody,
    controller: KinematicCharacterController,
    controller_output: KinematicCharacterControllerOutput,
    collider: Collider,
    attack: AttackCollider,
}

#[derive(Clone, Default, Debug, Component, Reflect)]
#[reflect(Component)]
struct Goblin;

#[derive(Clone, Default, Debug, Component, Reflect)]
#[reflect(Component)]
struct Slime;

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

fn spawn_enemies(mut commands: Commands) {
    commands
        .spawn(EnemyBundle {
            name: Enemy(Name("Goblin".to_string())),
            enemy: ActiveEntity {
                rotation: 1,
                velocity: Vec2::default(),
                current_state: EnemyStates::default(),
            },
            stats: Stats {
                health: Health::default(),
                armor: Armor(0),
                strength: Strength(10),
            },
            monster_type: Goblin,
            sprite: SpriteBundle {
                sprite: Sprite {
                    color: Color::hex("fc0315").unwrap(),
                    custom_size: Some(Vec2::new(40., 140.)),
                    ..default()
                },
                transform: Transform::from_xyz(150f32, 100f32, 10.),
                ..default()
            },
            rigid_body: RigidBody::KinematicVelocityBased,
            controller: KinematicCharacterController {
                slide: true,
                filter_flags: QueryFilterFlags::EXCLUDE_KINEMATIC
                    | QueryFilterFlags::EXCLUDE_SENSORS,
                ..default()
            },
            controller_output: KinematicCharacterControllerOutput::default(),
            collider: Collider::cuboid(20., 70.),
            attack: AttackCollider(None),
        })
        .insert(ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_KINEMATIC);

    commands
        .spawn(EnemyBundle {
            name: Enemy(Name("Slime".to_string())),
            enemy: ActiveEntity {
                rotation: 1,
                velocity: Vec2::default(),
                current_state: EnemyStates::default(),
            },
            stats: Stats {
                health: Health::default(),
                armor: Armor(0),
                strength: Strength(5),
            },
            monster_type: Slime,
            sprite: SpriteBundle {
                sprite: Sprite {
                    color: Color::hex("fc0303").unwrap(),
                    custom_size: Some(Vec2::splat(60f32)),
                    ..default()
                },
                transform: Transform::from_xyz(200f32, 100f32, 10.),
                ..default()
            },
            rigid_body: RigidBody::KinematicVelocityBased,
            controller: KinematicCharacterController {
                slide: true,
                filter_flags: QueryFilterFlags::EXCLUDE_KINEMATIC
                    | QueryFilterFlags::EXCLUDE_SENSORS,
                ..default()
            },
            controller_output: KinematicCharacterControllerOutput::default(),
            collider: Collider::ball(30.),

            attack: AttackCollider(None),
        })
        .insert(ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_KINEMATIC);
}

fn setup_player(mut commands: Commands) {
    commands
        .spawn(PlayerBundle {
            name: Player(Name("Player".to_string())),
            player: ActiveEntity {
                rotation: 1,
                velocity: Vec2::default(),
                current_state: PlayerStates::default(),
            },
            stats: Stats {
                health: Health::default(),
                armor: Armor(10),
                strength: Strength(10),
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
                    .insert(KeyCode::ShiftLeft, PlayerActions::Dash)
                    .insert(GamepadButtonType::East, PlayerActions::Dash)
                    .insert(MouseButton::Left, PlayerActions::Attack)
                    .set_gamepad(Gamepad { id: 0 })
                    .build(),
            },
            // rigid_body: RigidBody::Dynamic,
            // rigid_body: RigidBody::KinematicPositionBased,
            rigid_body: RigidBody::KinematicVelocityBased,
            controller: KinematicCharacterController {
                slide: true,
                autostep: None,
                filter_flags: QueryFilterFlags::EXCLUDE_KINEMATIC
                    | QueryFilterFlags::EXCLUDE_SENSORS,
                ..default()
            },
            controller_output: KinematicCharacterControllerOutput::default(),
            collider: Collider::capsule(Vec2::new(0., 40.), Vec2::new(0.0, -40.0), 30.),

            attack: AttackCollider(None),
        })
        .insert(ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_KINEMATIC);
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
                                         // info!("|||||||||||||||||||||||SLIDING|||||||||||||||||||||||");
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
                    continue;
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

fn player_attack(
    mut commands: Commands,
    time: Res<Time>,
    mut controller_query: Query<
        (
            Entity,
            &ActionState<PlayerActions>,
            &mut AttackCollider,
            &Transform,
            &mut ActiveEntity<PlayerStates>,
            &mut KinematicCharacterController,
            Option<&KinematicCharacterControllerOutput>,
        ),
        With<Player>,
    >,
    mut time_attack: Local<TimeAttack>, //TODO: attach to weapon
) {
    let (p_entity, action_state, mut attack_collider, t, _, _, _) = controller_query.single_mut();
    let is_attack = action_state.just_pressed(PlayerActions::Attack);
    if let Some(collider_entity) = attack_collider.0 {
        time_attack.0.tick(time.delta());
        if time_attack.0.finished() {
            commands
                .entity(p_entity)
                .remove_children(&[collider_entity]);
            commands.entity(collider_entity).despawn();
            attack_collider.0 = None;
        }
        return;
    }
    if !is_attack {
        return;
    }
    time_attack.0.reset();
    let entity = commands
        .spawn((
            TransformBundle::from_transform(Transform::from_xyz(50., 0., 0.)),
            // TransformBundle::from_transform(t.clone()),
            Collider::cuboid(10., 20.),
            Sensor,
            ActiveEvents::COLLISION_EVENTS, // .insert(ActiveEvents::COLLISION_EVENTS)
        ))
        .id();
    attack_collider.0 = Some(entity);
    commands.entity(p_entity).add_child(entity);
}

fn attack_checker(// mut q: Query<&mut Transform, (With<AttackCollider>, Without<Player>)>,
    // p_q: Query<&Transform, With<Player>>,
) {
    // let mut attack_transform = q.single_mut();
    // let player_transform = p_q.single();

    // attack_transform.translation = player_transform.translation;
}

fn isset_collider(query: Query<&AttackCollider, With<Player>>) -> bool {
    let attack = query.single();
    match attack.0 {
        Some(_) => true,
        None => false,
    }
}

fn move_enemy_goblin(
    time: Res<Time>,
    rapier_config: Res<RapierConfiguration>,
    mut controller_query: Query<
        (
            // &Goblin,
            &mut ActiveEntity<EnemyStates>,
            &mut KinematicCharacterController,
            &Transform,
            Option<&KinematicCharacterControllerOutput>,
        ),
        With<Goblin>,
    >,
    player_q: Query<&Transform, With<Player>>,
) {
    let player_pos = player_q.single();
    for (mut enemy, mut controller, enemy_pos, controller_output) in controller_query.iter_mut() {
        let grounded = match controller_output {
            Some(out) => out.grounded,
            None => false,
        };

        let dt = time.delta_seconds();
        let speed = 47.0;
        let mut instant_acceleration = Vec2::ZERO;
        let mut instant_velocity = enemy.velocity;

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
            }
        }

        let mut rotation = 1.;
        if enemy_pos.translation.x > player_pos.translation.x {
            rotation = -1.;
        }

        // TODO: logic move for enemies
        instant_velocity += Vec2::new(rotation * speed, 0.);
        instant_velocity = instant_velocity.clamp(Vec2::splat(-1000.0), Vec2::splat(1000.0));
        enemy.velocity = (instant_acceleration * dt) + instant_velocity;
        let translation = controller.translation.unwrap_or(Vec2::new(0., 0.)) + enemy.velocity * dt;
        controller.translation = Some(translation);
    }
}

fn move_enemy_slime(
    time: Res<Time>,
    rapier_config: Res<RapierConfiguration>,
    mut controller_query: Query<
        (
            // &Goblin,
            &mut ActiveEntity<EnemyStates>,
            &mut KinematicCharacterController,
            &Transform,
            Option<&KinematicCharacterControllerOutput>,
        ),
        With<Slime>,
    >,
    player_q: Query<&Transform, With<Player>>,
) {
    let player_pos = player_q.single();
    for (mut enemy, mut controller, enemy_pos, controller_output) in controller_query.iter_mut() {
        let grounded = match controller_output {
            Some(out) => out.grounded,
            None => false,
        };

        let dt = time.delta_seconds();
        let speed = 47.0;
        let mut instant_acceleration = Vec2::ZERO;
        let mut instant_velocity = enemy.velocity;
        let jump_impulse = 1000f32;
        // physics simulation
        if grounded {
            // friction
            instant_velocity.x *= 0.9;
        } else {
            // friction in jump
            instant_velocity.x *= 0.8;
            // gravity
            if !grounded {
                instant_acceleration += Vec2::Y * rapier_config.gravity;
            }
        }
        let mut y = 0f32;
        if grounded {
            y = jump_impulse;
        }

        let mut rotation = 1.;
        if enemy_pos.translation.x > player_pos.translation.x {
            rotation = -1.;
        }

        // TODO: logic move for enemies
        instant_velocity += Vec2::new(rotation * speed, y);
        instant_velocity = instant_velocity.clamp(Vec2::splat(-1000.0), Vec2::splat(1000.0));
        enemy.velocity = (instant_acceleration * dt) + instant_velocity;
        let translation = controller.translation.unwrap_or(Vec2::new(0., 0.)) + enemy.velocity * dt;
        controller.translation = Some(translation);
    }
}

fn player_collision(
    // mut cmd: Commands,
    q: Query<(&KinematicCharacterControllerOutput, &Transform), With<Player>>,
    mut stop_jump: EventWriter<StopJump>,
    mut slide: EventWriter<SlideEvent>,
) {
    // info!("Start collision detect");
    for (out, transform) in q.iter() {
        const HEIGHT_PLAYER: f32 = 68.; // size height / 2 -2
        const WIDTH_PLAYER: f32 = 28.; // sizwe width / 2 - 2
        let x_left_player = transform.translation.x - WIDTH_PLAYER;
        let x_right_player = transform.translation.x + WIDTH_PLAYER;

        let y_top_player = transform.translation.y + HEIGHT_PLAYER;
        let y_down_player = transform.translation.y - HEIGHT_PLAYER;

        // info!("Player transform {:?}", transform);
        for collision in &out.collisions {
            if y_top_player < collision.toi.witness1.y {
                // up collision player
                // so that the hero starts falling when he hits the ceiling
                if collision.toi.witness1.x > x_left_player
                    && collision.toi.witness1.x < x_right_player
                {
                    // angel player not top
                    stop_jump.send_default();
                }
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

#[derive(Resource, Debug, Reflect)]
#[reflect(Resource)]
struct BeforeCollider(Option<Entity>);

impl Default for BeforeCollider {
    fn default() -> Self {
        Self(None)
    }
}

// TODO: dont work collision events with Rigid body velocity based
fn sensor_event(
    mut commands: Commands,
    mut q: Query<(Entity, &mut Health, &Enemy), With<Enemy>>,
    mut collision_events: EventReader<CollisionEvent>,
) {
    for (enemy_entity, mut health_enemy, name_enemy) in q.iter_mut() {
        // info!("{:?}", name_enemy);
        for collision_event in collision_events.iter() {
            match collision_event {
                CollisionEvent::Started(entity_event, sensor_entity, type_entity) => {
                    info!("[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[\n");
                    info!("{:?}", entity_event);
                    if entity_event.eq(&enemy_entity) {
                        health_enemy.0 -= 10;
                    }
                    info!("{:?}", type_entity);
                    info!("{:?}", name_enemy);
                    info!("]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]\n");
                }
                CollisionEvent::Stopped(entity_event, sensor_entity, type_entity) => (),
            }
            // println!("Received collision event: {:?}", collision_event);
        }
    }
}
