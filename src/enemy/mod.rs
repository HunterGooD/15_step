mod goblin;
mod slime;

// TODO: Think about it:
/**
 *
 * A general function for movement, where the opponent will have a distance and attack offset based on which he stops at a certain attack distance.
 * Calls the attack method.
 * Also, if there are any movement specifications,
 * they can be added in a separate subsystem or values can be assigned to the fields of the enemy structure
 */
use crate::enemy::goblin::GoblinsPlugin;
use crate::enemy::slime::SlimesPlugin;
use crate::entities::*;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((SlimesPlugin, GoblinsPlugin));
        // .add_systems(Update, sensor_event.run_if(in_state(GameState::InGame)));
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
