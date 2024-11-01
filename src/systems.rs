use bevy::prelude::{Commands, Entity, Query, ResMut};
use salva3d::{math::Point, object::Fluid};

use crate::{fluid::{FluidDensity, FluidParticlePositions, SalvaFluidHandle}, plugin::SalvaContext};

pub fn init_fluids(
    mut commands: Commands,
    query: Query<(
        Entity, &FluidParticlePositions, Option<&FluidDensity>
    )>,
    mut salva_cxt: ResMut<SalvaContext>
) {
    for (entity, particle_positions, density) in query.iter() {
        let density = density.map_or_else(|| 1000.0, |d| d.density0);
        let particle_positions: Vec<_> = particle_positions.positions
            .iter().map(|v| Point::new(v.x, v.y, v.z))
            .collect();
        let salva_fluid = Fluid::new(particle_positions, salva_cxt.liquid_world.particle_radius(), density);
        let fluid_handle = salva_cxt.liquid_world.add_fluid(salva_fluid);
        commands.get_entity(entity).unwrap().insert(SalvaFluidHandle(fluid_handle));
    }
}
