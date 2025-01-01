use bevy::prelude::Component;
use salva::{object::FluidHandle, solver::NonPressureForce};
use crate::math::{Real, Vect};

#[derive(Component)]
pub struct SalvaFluidHandle(pub FluidHandle);

#[derive(Component)]
pub struct FluidParticlePositions {
    pub positions: Vec<Vect>,
}

/// The rest density of a fluid (default 1000.0)
#[derive(Component)]
pub struct FluidDensity {
    pub density0: Real,
}

impl Default for FluidDensity {
    fn default() -> Self {
        Self { density0: 1000.0 }
    }
}

#[derive(Component)]
pub struct FluidNonPressureForces(pub Vec<Box<dyn NonPressureForce>>);

#[derive(Component)]
pub struct AppendNonPressureForces(pub Vec<Box<dyn NonPressureForce>>);

#[derive(Component)]
pub struct RemoveNonPressureForcesAt(pub Vec<usize>);