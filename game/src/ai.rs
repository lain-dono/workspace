use bevy::prelude::*;

#[derive(Component)]
pub struct Actor {
    pub motives: Vec<Motive>,
}

pub struct Motive {
    pub current: f32,
}

pub struct Human {
    pub name: String,
    pub nick: String,
    pub family: String,
}
