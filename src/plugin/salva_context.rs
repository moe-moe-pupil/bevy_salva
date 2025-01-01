//TODO: salva context, default read/write, SalvaEntity query data

use salva::math::Vector;
use bevy::prelude::{Entity, Resource};
use salva::LiquidWorld;
use std::collections::HashMap;
use salva::object::FluidHandle;

#[cfg(feature = "rapier")]
use salva::integrations::rapier::ColliderCouplingSet;
#[cfg(feature = "rapier")]
use bevy_rapier::plugin::RapierContext;

#[derive(Resource)]
pub struct SalvaContext {
    pub liquid_world: LiquidWorld,
    #[cfg(feature = "rapier")]
    pub coupling: ColliderCouplingSet,
    pub entity2fluid: HashMap<Entity, FluidHandle>,
}

impl SalvaContext {
    #[cfg(feature = "rapier")]
    pub fn step(&mut self, dt: f32, gravity: &Vector<f32>, rapier_context: &mut RapierContext) {
        self.liquid_world.step_with_coupling(
            dt,
            gravity,
            &mut self
                .coupling
                .as_manager_mut(&rapier_context.colliders, &mut rapier_context.bodies),
        );
    }


    #[cfg(not(feature = "rapier"))]
    pub fn step(&mut self, dt: f32, gravity: &Vector<f32>) {
        self.liquid_world.step(
            dt,
            gravity
        );
    }
}