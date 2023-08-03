use crate::entities::*;
use crate::enemy::*;
use crate::GameState;
use crate::player::Player;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;


pub struct GoblinsPlugin;

impl Plugin for GoblinsPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(OnEnter(GameState::InGame), spawn_goblins)
        .add_systems(
            Update,
            (
                move_goblins
            )
                .run_if(in_state(GameState::InGame)),
        );
    }
}

#[derive(Clone, Default, Debug, Component, Reflect)]
#[reflect(Component)]
pub struct Goblin;

fn spawn_goblins(mut commands: Commands) {
     commands
        .spawn(EnemyBundle {
            name: Enemy(Name::new("Goblin")),
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

}

fn move_goblins(
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