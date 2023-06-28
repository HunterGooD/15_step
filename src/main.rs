use std::time::Duration;

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    window::{PresentMode, WindowMode},
};
use bevy_ecs_tilemap::prelude::*;
use bevy_parallax::{
    LayerData, LayerSpeed, ParallaxCameraComponent, ParallaxMoveEvent, ParallaxPlugin,
    ParallaxResource,
};

use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier2d::{prelude::*, na::ComplexField};
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
                // mode: WindowMode::Fullscreen,
                ..default()
            }),
            ..default()
        }))
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(bevy::diagnostic::SystemInformationDiagnosticsPlugin::default())
        .add_plugin(bevy::diagnostic::EntityCountDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin)
        .add_plugin(WorldInspectorPlugin::default())
        .add_plugin(TilemapPlugin)
        .add_plugin(ParallaxPlugin)
        .insert_resource(ParallaxResource {
            layer_data: vec![
                LayerData {
                    speed: LayerSpeed::Horizontal(0.9),
                    path: "tiles/forest/background/layer_1.png".to_string(),
                    tile_size: Vec2::new(320.0, 180.0),
                    cols: 1,
                    rows: 1,
                    scale: 17.5,
                    z: 0.0,
                    ..Default::default()
                },
                LayerData {
                    speed: LayerSpeed::Horizontal(0.6),
                    path: "tiles/forest/background/layer_2.png".to_string(),
                    tile_size: Vec2::new(320.0, 180.0),
                    cols: 1,
                    rows: 1,
                    scale: 9.5,
                    z: 1.0,
                    ..Default::default()
                },
                LayerData {
                    speed: LayerSpeed::Horizontal(0.1),
                    path: "tiles/forest/background/layer_3.png".to_string(),
                    tile_size: Vec2::new(320.0, 180.0),
                    cols: 1,
                    rows: 1,
                    scale: 8.5,
                    z: 2.0,
                    ..Default::default()
                },
            ],
            ..Default::default()
        })
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0)) // create world rapier physics
        .insert_resource(RapierConfiguration {
            gravity: Vec2::new(0.0, -2000.0),
            ..default()
        })
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(InputManagerPlugin::<PlayerActions>::default()) // player actions for buttons
        .add_plugin(InputManagerPlugin::<CameraActions>::default())
        .add_startup_systems((setup_graphics, setup_map, setup_player).chain())
        .add_system(camera_settings)
        .add_system(pan_camera)
        .add_system(move_player)
        .add_system(log_states)
        .add_system(parallax_move)
        .run();
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
enum PlayerActions {
    Move,
    Jump,
    Dash,
}

// for debug
#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
enum CameraActions {
    Zoom,
    PanLeft,
    PanRight,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Default)]
enum PlayerStates {
    #[default]
    Idle,
    Run,
    Jump,
    Fall,
}

#[derive(Clone, Default, Debug, Component)]
struct Name(String);

#[derive(Clone, Default, Debug, Component)]
struct Player {
    rotation: i8,
    velocity: Vec2,
    current_state: PlayerStates,
}

const MAX_JUMP: u8 = 2;

#[derive(Resource, Debug)]
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

const DASH_FRAMES: u8 = 7;
const DASH_SPEED_FACTOR: f32 = 0.1;

#[derive(Resource, Debug)]
struct DashInfo {
    cooldown_time: Timer,
    evade_time: Timer,
    is_used: bool,
    frames_count: u8,
}

impl Default for DashInfo {
    fn default() -> Self {
        let mut cooldown_timer = Timer::new(Duration::from_millis(350), TimerMode::Once);
        cooldown_timer.tick(Duration::from_secs(1)); // finished on init
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
    name: Name,
    player: Player,
    sprite: SpriteBundle,
    input: InputManagerBundle<PlayerActions>,
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
                .insert(MouseWheelDirection::Left, CameraActions::PanLeft)
                .insert(MouseWheelDirection::Right, CameraActions::PanRight)
                .build(),
            ..default()
        })
        .insert(ParallaxCameraComponent);
}

fn parallax_move(
    action_q: Query<(&ActionState<PlayerActions>, &Player, &Transform), With<Player>>,
    camera_q: Query<&Transform, With<Camera2d>>,
    mut move_event_writer: EventWriter<ParallaxMoveEvent>,
) {
    let (action_state, player, transform) = action_q.single();
    let transform_cam = camera_q.single();
    info!("Player {:?}", transform);
    info!("Camera {:?}", transform_cam);
    let axis_vector = action_state
        .clamped_axis_pair(PlayerActions::Move)
        .unwrap()
        .x();

    let mut speed_camera = 9.5;
    //TODO: valocity for camera
    if (transform_cam.translation.x.abs() - transform.translation.x.abs()).abs() < 150.0 {
        speed_camera = 12.5;
    } else if (transform_cam.translation.x.abs() - transform.translation.x.abs()).abs() > 150.0 {
        speed_camera = 4.5;
    }

    if axis_vector > 0.01 {
        move_event_writer.send(ParallaxMoveEvent {
            camera_move_speed: Vec2::new(speed_camera, 0.0),
        });
    } else if axis_vector < -0.01 {
        move_event_writer.send(ParallaxMoveEvent {
            camera_move_speed: Vec2::new(-speed_camera, 0.0),
        });
    }

    match player.current_state {
        PlayerStates::Jump => move_event_writer.send(ParallaxMoveEvent {
            camera_move_speed: Vec2::new(0.0, 3.0),
        }),
        PlayerStates::Fall => move_event_writer.send(ParallaxMoveEvent {
            camera_move_speed: Vec2::new(0.0, -3.0),
        }),
        _ => (),
    }
}

fn camera_settings(
    mut q: Query<(&mut OrthographicProjection, &ActionState<CameraActions>), With<Camera2d>>,
) {
    const CAMERA_ZOOM_RATE: f32 = 0.05;

    let (mut projection, action_state) = q.single_mut();
    let zoom_delta = action_state.value(CameraActions::Zoom);

    projection.scale *= 1. - zoom_delta * CAMERA_ZOOM_RATE;
}

fn pan_camera(mut query: Query<(&mut Transform, &ActionState<CameraActions>), With<Camera2d>>) {
    const CAMERA_PAN_RATE: f32 = 10.;

    let (mut camera_transform, action_state) = query.single_mut();

    // When using the `MouseWheelDirection` type, mouse wheel inputs can be treated like simple buttons
    if action_state.pressed(CameraActions::PanLeft) {
        camera_transform.translation.x -= CAMERA_PAN_RATE;
    }

    if action_state.pressed(CameraActions::PanRight) {
        camera_transform.translation.x += CAMERA_PAN_RATE;
    }
}

fn setup_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    let texture_handle: Handle<Image> = asset_server.load("tiles/forest/tileset.png"); // 31 x 22 tiles
    let map_size = TilemapSize { x: 500, y: 1 };

    let mut tile_storage = TileStorage::empty(map_size);
    let tilemap_entity = commands.spawn_empty().id();

    fill_tilemap(
        TileTextureIndex(1),
        map_size,
        TilemapId(tilemap_entity),
        &mut commands,
        &mut tile_storage,
    );

    let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
    let grid_size = tile_size.into();
    let map_type = TilemapType::default();

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle.clone()),
        tile_size,
        transform: Transform::from_xyz(-4976., -374., 1.).with_scale(Vec3::splat(3.)),
        ..Default::default()
    });

    commands
        .spawn(Collider::cuboid(5000., 50.))
        .insert(RigidBody::Fixed)
        .insert(TransformBundle::from_transform(Transform::from_xyz(
            0., -400., 0.,
        )));
}

fn setup_player(mut commands: Commands) {
    commands.spawn(PlayerBundle {
        name: Name("Player".to_string()),
        player: Player {
            rotation: 1,
            velocity: Vec2::default(),
            current_state: PlayerStates::default(),
        },
        sprite: SpriteBundle {
            sprite: Sprite {
                color: Color::hex("969696").unwrap(),
                custom_size: Some(Vec2::new(60., 140.)),
                ..Default::default()
            },
            transform: Transform::from_xyz(0., 400., 10.),
            ..Default::default()
        },
        input: InputManagerBundle::<PlayerActions> {
            action_state: ActionState::default(),
            input_map: InputMap::default()
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
            ..default()
        },
        controller_output: KinematicCharacterControllerOutput::default(),
        collider: Collider::cuboid(30., 70.),
    });
}

fn log_states(q: Query<&Player, With<Player>>) {
    let player = q.single();
    // info!("{:?}", player.current_state);
}

fn move_player(
    time: Res<Time>,
    rapier_config: Res<RapierConfiguration>,
    mut jump_info: Local<JumpInfo>,
    mut dash_info: Local<DashInfo>,
    mut controller_query: Query<
        (
            &ActionState<PlayerActions>,
            &mut Player,
            &mut KinematicCharacterController,
            Option<&KinematicCharacterControllerOutput>,
        ),
        With<Player>,
    >,
) {
    let (action_state, mut player, mut controller, controller_output) =
        controller_query.single_mut();

    let grounded = match controller_output {
        Some(out) => out.grounded,
        None => false,
    };

    let dt = time.delta_seconds();

    // TODO: global variable
    let speed = 75.0;
    let jump_impulse = 1000.0;
    let mut instant_acceleration = Vec2::ZERO;
    let mut instant_velocity = player.velocity;

    // physics simulation
    if grounded {
        // friction
        instant_velocity.x *= 0.85;
    } else {
        // friction in jump
        instant_velocity.x *= 0.9;
        // gravity
        if !grounded {
            instant_acceleration += Vec2::Y * rapier_config.gravity;
        }
    }

    let axis_vector = action_state
        .clamped_axis_pair(PlayerActions::Move)
        .unwrap()
        .x();
    if axis_vector != 0. {
        player.rotation = if (axis_vector * 9.) < 0. { -1 } else { 1 };
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
    controller.translation =
        Some(controller.translation.unwrap_or(Vec2::new(0., 0.)) + player.velocity * dt);
}
