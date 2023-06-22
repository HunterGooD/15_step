use std::time::Duration;

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    window::{PresentMode, WindowMode},
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
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
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0)) // create world rapier physics
        .insert_resource(RapierConfiguration {
            gravity: Vec2::new(0.0, -2000.0),
            ..default()
        })
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(InputManagerPlugin::<PlayerActions>::default()) // player actions for buttons
        .add_startup_systems((setup_graphics, setup_map, setup_player))
        // .insert_resource(JumpInfo {
        //     count: 1,
        //     time_up: Timer::new(Duration::from_millis(500), TimerMode::Once),
        // })
        .add_system(move_player)
        .add_system(log_states)
        .run();
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
enum PlayerActions {
    Move,
    Jump,
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
        Self { count: 0, time_up: Timer::new(Duration::from_millis(500), TimerMode::Once) }
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
    commands.spawn(Camera2dBundle::default());
}

fn setup_map(mut commands: Commands) {
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
            transform: Transform::from_xyz(0., 400., 0.),
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
    info!("{:?}", player.current_state);
}

fn move_player(
    time: Res<Time>,
    rapier_config: Res<RapierConfiguration>,
    mut jump_info: Local<JumpInfo>,
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

    let mut y = 0.;
    for action in action_state.get_just_pressed() {
        match action {
            PlayerActions::Jump => {
                if jump_info.count >= 2 {
                    return;
                }
                // instant_acceleration.y = 1.;
                instant_velocity.y = 1.;
                y = jump_impulse;
                jump_info.time_up.reset();
                // jump_info.time_up.tick(time.delta());

                jump_info.count += 1;
                player.current_state = PlayerStates::Jump;
            }
            _ => (),
        }
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

    instant_velocity += Vec2::new(axis_vector * speed, y);
    instant_velocity = instant_velocity.clamp(Vec2::splat(-1000.0), Vec2::splat(1000.0));
    player.velocity = (instant_acceleration * dt) + instant_velocity;
    controller.translation = Some(player.velocity * dt);
}
