use std::ops::{Deref, DerefMut};

use bevy::{math::Vec3, prelude::Component};
use salva3d::{math::{Point, Real}, object::FluidHandle};

#[derive(Component)]
pub struct SalvaFluidHandle(pub FluidHandle);

#[derive(Component)]
pub struct Fluid {
    raw: salva3d::object::Fluid
}

impl Fluid {
    pub fn new(
        particle_positions: Vec<Vec3>,
        particle_radius: Real, // XXX: remove this parameter since it is already defined by the liquid world.
        density0: Real,
    ) -> Self {
        let salva_particle_positions = particle_positions.iter()
            .map(|pt| { Point::new(pt.x, pt.y, pt.z) })
            .collect();

        Self {
            raw: salva3d::object::Fluid::new(salva_particle_positions, particle_radius, density0)
        }
    }
}

impl Deref for Fluid {
    type Target = salva3d::object::Fluid;

    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}

impl DerefMut for Fluid {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.raw
    }
}
