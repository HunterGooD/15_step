mod slime;
mod goblin;

use crate::GameState;
use crate::enemy::slime::SlimesPlugin;
use crate::enemy::goblin::GoblinsPlugin;
use crate::entities::*;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            SlimesPlugin,
            GoblinsPlugin,
        )).add_systems(Update, sensor_event.run_if(in_state(GameState::InGame)));
    }
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
struct Enemy(Name);


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

#[derive(Resource, Debug, Reflect)]
#[reflect(Resource)]
struct BeforeCollider(Option<Entity>);

impl Default for BeforeCollider { // add for dont hit from one entity more
    fn default() -> Self {
        Self(None)
    }
}

// TODO: dont work collision events with Rigid body velocity based
fn sensor_event(
    // mut commands: Commands,
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
                    info!("{:?}", sensor_entity);
                    if entity_event.eq(&enemy_entity) {
                        health_enemy.0 -= 10;
                    }
                    info!("{:?}", type_entity);
                    info!("{:?}", name_enemy);
                    info!("]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]\n");
                }
                CollisionEvent::Stopped(_, _, _) => (),
            }
        }
    }
}