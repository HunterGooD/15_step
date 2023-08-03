use crate::entities::*;
use crate::enemy::*;
use crate::GameState;
use crate::player::Player;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

pub struct SlimesPlugin;

impl Plugin for SlimesPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(OnEnter(GameState::InGame), spawn_slimes)
        .add_systems(
            Update,
            (
                move_slime
            )
                .run_if(in_state(GameState::InGame)),
        );
    }
}

#[derive(Clone, Default, Debug, Component, Reflect)]
#[reflect(Component)]
pub struct Slime;

fn spawn_slimes(mut commands: Commands) {
     commands
        .spawn(EnemyBundle {
            name: Enemy(Name::new("Slime")),
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

fn move_slime(
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