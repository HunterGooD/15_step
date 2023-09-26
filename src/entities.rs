use bevy::prelude::*;

// **********************************************************  STATS
// add speed how stats
// stats for weapon > attack damage, attack speed, attack range, attack interrupt, attack knockback
// Strength * attack damage = true damage
#[derive(Clone, Debug, Component, Reflect)]
#[reflect(Component)]
pub struct Health(pub i32);

impl Default for Health {
    fn default() -> Self {
        Self(100)
    }
}

#[derive(Clone, Default, Debug, Component, Reflect)]
#[reflect(Component)]
pub struct Armor(pub i32);

#[derive(Clone, Default, Debug, Component, Reflect)]
#[reflect(Component)]
pub struct Strength(pub i32);

// *********************************************************  END STATS

#[derive(Bundle, Default)]
pub struct Stats {
    pub health: Health,
    pub armor: Armor,
    pub strength: Strength,
}

#[derive(Clone, Default, Debug, Component, Reflect)]
#[reflect(Component)]
pub struct AttackCollider(pub Option<Entity>);

#[derive(Clone, Default, Debug, Component, Reflect)]
#[reflect(Component)]
pub struct ActiveEntity<T: Default> {
    pub rotation: i8,
    pub velocity: Vec2,
    pub current_state: T,
}

#[derive(Clone, Default, Debug, Component, Reflect)]
#[reflect(Component)]
pub struct Damage(pub isize);

#[derive(Clone, Default, Debug, Component, Reflect)]
#[reflect(Component)]
pub struct ModifyStat {
    pub time: Timer,
    pub speed_scale: isize,
    pub effect: Color,
}

pub struct DoTStat {
    pub time: Timer,
    pub tick_dot: Timer,
    pub damage: Damage,
    pub stuck: isize,
    // pub effect_type
}

pub enum StatType {
    AttackDamage,
    MoveSpeed,
    
}


pub enum StatModificationType {
    Percentage,
    Numerical,
    Absolute,
    None,
}

