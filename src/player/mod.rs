use crate::entities::*;
use crate::GameState;
use crate::loading::PlayerTexture;

use bevy::{prelude::*, utils::Duration};
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::prelude::*;
use leafwing_input_manager::InputManagerBundle;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0), // create world rapier physics
            RapierDebugRenderPlugin::default(),
            InputManagerPlugin::<PlayerActions>::default(), // player actions for buttons
            InputManagerPlugin::<CameraActions>::default(),
        ))
        .insert_resource(RapierConfiguration {
            gravity: Vec2::new(0.0, -2000.0),
            ..default()
        })
        .add_event::<StopJump>()
        .add_event::<SlideEvent>()
        .add_systems(OnEnter(GameState::InGame), (spawn_player, spawn_camera))
        .add_systems(
            Update,
            (
                move_player,
                player_attack,
                follow,
                camera_settings,
                player_collision,
            )
                .run_if(in_state(GameState::InGame)),
        );
    }
}

const DASH_FRAMES: u8 = 7;
const DASH_SPEED_FACTOR: f32 = 0.1;
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

#[derive(Clone, Default, Debug, Component, Reflect)]
#[reflect(Component)]
pub struct Player(pub Name);

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Default, Reflect)]
enum PlayerStates {
    #[default]
    Idle,
    Run,
    Jump,
    Fall,
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
fn spawn_camera(mut commands: Commands) {
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

//TODO: create abstract entity bundle
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

fn spawn_player(mut commands: Commands, texture: Res<PlayerTexture>) {
    commands
        .spawn(PlayerBundle {
            name: Player(Name::new("Player")),
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
                texture: texture.sprite.clone(),
                transform: Transform::from_xyz(-100., 400., 10.).with_scale(Vec3::splat(4.)), // 50 x 37  * 4 200 x 148
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
            collider: Collider::capsule(Vec2::new(0., 8.), Vec2::new(0.0, -11.0), 7.),

            attack: AttackCollider(None),
        })
        .insert(ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_KINEMATIC);
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
        .unwrap_or_default()
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
        info!("Jsajdjasdjaj");
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
    let (p_entity, action_state, mut attack_collider, _, _, _, _) = controller_query.single_mut();
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
            TransformBundle::from_transform(Transform::from_xyz(5., 0., 0.)),
            // TransformBundle::from_transform(t.clone()),
            Collider::cuboid(5., 10.),
            Sensor,
            ActiveEvents::COLLISION_EVENTS, // .insert(ActiveEvents::COLLISION_EVENTS)
        ))
        .id();
    attack_collider.0 = Some(entity);
    commands.entity(p_entity).add_child(entity);
}

// TODO: camera.rs for 1 function ?
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

fn player_collision(
    // mut cmd: Commands,
    q: Query<(&KinematicCharacterControllerOutput, &Transform), With<Player>>,
    mut stop_jump: EventWriter<StopJump>,
    mut slide: EventWriter<SlideEvent>,
) {
    // info!("Start collision detect");
    for (out, transform) in q.iter() {
        const HEIGHT_PLAYER: f32 = 58.; // size height / 2 -2
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
                if collision.toi.witness1.x > transform.translation.x {
                    slide.send(SlideEvent(WallPosition::Right));
                } else {
                    slide.send(SlideEvent(WallPosition::Left));
                }
            }

            if y_down_player > collision.toi.witness1.y {
                // ground
            }


            // info!("{:?}", collision);
        }
    }
    // info!("End collision detect");
}
