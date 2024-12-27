use bevy::{math::Vec3, prelude::Component};
use salva3d::{math::Real, object::FluidHandle, solver::NonPressureForce};

#[derive(Component)]
pub struct SalvaFluidHandle(pub FluidHandle);

#[derive(Component)]
pub struct FluidParticlePositions {
    pub positions: Vec<Vec3>,
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
