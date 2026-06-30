use bevy::prelude::*;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct RTCamera;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct RTDisplay;

#[derive(Component, Debug, Default, Reflect, Clone, Copy)]
#[require(RTMass(1.0), Transform)]
pub struct RTObject(pub u32);

#[derive(Component, Reflect)]
pub struct RTMass(pub f32);
