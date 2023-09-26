use bevy::prelude::*;

#[derive(Event, Debug)]
pub struct AttackEvent {
    pub entity: Entity,
    pub damage: isize,
    pub effects: Vec<isize>,
}
