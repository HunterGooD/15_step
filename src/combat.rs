use crate::{entities::{Health}, events::AttackEvent, GameState, InGameState};
use bevy::prelude::*;

// plugin for damage, debuff, buff and other
// pub struct CombatPlugin;

// impl Plugin for CombatPlugin {
//     fn build(&self, app: &mut App) {
//         app.add_systems(
//             Update,
//             (damage_hit, debuff_effect)
//                 .run_if(in_state(GameState::InGame))
//                 .run_if(in_state(InGameState::Play)),
//         );
//     }
// }

// fn damage_hit(mut cmd: Commands, mut query: Query<&mut Health/*Speed */>, event: EventReader<AttackEvent>) {
//     println!("123123123123123")
// }

// fn debuff_effect(hp: Query<&mut Health/*Speed */>, debuff: Query<>) {
//     println!("asdasdasdasd")
// }