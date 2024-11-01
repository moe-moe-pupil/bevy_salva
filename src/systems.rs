use bevy::prelude::{Commands, Entity, Query, ResMut};
use salva3d::{math::Point, object::Fluid};

use crate::{fluid::{FluidDensity, FluidNonPressureForces, FluidParticlePositions, SalvaFluidHandle}, plugin::SalvaContext};

pub fn init_fluids(
    mut commands: Commands,
    query: Query<(
        Entity,
        &FluidParticlePositions,
        Option<&FluidDensity>,
        Option<&mut FluidNonPressureForces>
    )>,
    mut salva_cxt: ResMut<SalvaContext>
) {
    for (entity, particle_positions, density, mut nonpressure_forces) in query.iter() {
        let density = density.map_or_else(|| 1000.0, |d| d.density0);
        let particle_positions: Vec<_> = particle_positions.positions
            .iter().map(|v| Point::new(v.x, v.y, v.z))
            .collect();
        let mut salva_fluid = Fluid::new(particle_positions, salva_cxt.liquid_world.particle_radius(), density);
        
        let mut entity_cmd = commands.get_entity(entity).unwrap();
        if let Some(nonpressure_forces) = nonpressure_forces {
            
        }
        
        let fluid_handle = salva_cxt.liquid_world.add_fluid(salva_fluid);
        entity_cmd.insert(SalvaFluidHandle(fluid_handle));
    }
}
