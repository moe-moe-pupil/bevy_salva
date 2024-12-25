use bevy::prelude::{Changed, Commands, Entity, Query, RemovedComponents, Res, ResMut, Time, Without};
use salva3d::{math::Point, object::Fluid};
use salva3d::math::Vector;
use crate::{
    fluid::{FluidDensity, FluidNonPressureForces, FluidParticlePositions, SalvaFluidHandle},
    plugin::{AppendNonPressureForces, RemoveNonPressureForcesAt, SalvaContext},
};

pub fn init_fluids(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &FluidParticlePositions,
            Option<&FluidDensity>,
            Option<&mut FluidNonPressureForces>,
        ),
        Without<SalvaFluidHandle>,
    >,
    mut salva_cxt: ResMut<SalvaContext>,
) {
    for (entity, particle_positions, density, nonpressure_forces) in query.iter_mut() {
        let density = density.map_or_else(|| 1000.0, |d| d.density0);
        let particle_positions: Vec<_> = particle_positions
            .positions
            .iter()
            .map(|v| Point::new(v.x, v.y, v.z))
            .collect();
        let mut salva_fluid = Fluid::new(
            particle_positions,
            salva_cxt.liquid_world.particle_radius(),
            density,
        );

        let mut entity_cmd = commands.get_entity(entity).unwrap();
        if let Some(mut nonpressure_forces) = nonpressure_forces {
            salva_fluid
                .nonpressure_forces
                .append(&mut nonpressure_forces.0);
        }

        let fluid_handle = salva_cxt.liquid_world.add_fluid(salva_fluid);
        entity_cmd.insert(SalvaFluidHandle(fluid_handle));

        salva_cxt.entity2fluid.insert(entity, fluid_handle);
    }
}

pub fn apply_nonpressure_force_changes(
    mut append_q: Query<
        (&SalvaFluidHandle, &mut AppendNonPressureForces),
        Changed<AppendNonPressureForces>,
    >,
    mut remove_at_q: Query<
        (&SalvaFluidHandle, &mut RemoveNonPressureForcesAt),
        Changed<RemoveNonPressureForcesAt>,
    >,
    mut salva_cxt: ResMut<SalvaContext>,
) {
    for (handle, mut appends) in append_q.iter_mut() {
        salva_cxt
            .liquid_world
            .fluids_mut()
            .get_mut(handle.0)
            .unwrap()
            .nonpressure_forces
            .append(&mut appends.0);
    }

    for (handle, mut removals) in remove_at_q.iter_mut() {
        let nonpressure_forces = &mut salva_cxt
            .liquid_world
            .fluids_mut()
            .get_mut(handle.0)
            .unwrap()
            .nonpressure_forces;

        for i in removals.0.iter() {
            nonpressure_forces.remove(*i);
        }
        removals.0.clear();
    }
}

pub fn sync_removals(
    mut removed_particle_positions: RemovedComponents<FluidParticlePositions>,
    mut removed_fluids: RemovedComponents<SalvaFluidHandle>,
    mut salva_cxt: ResMut<SalvaContext>,
) {
    //remove fluids whos entities had their salva fluid handle or fluid particle components removed
    for entity in removed_fluids
        .read()
        .chain(removed_particle_positions.read())
    {
        let handle = *salva_cxt.entity2fluid.get(&entity).unwrap();
        salva_cxt.liquid_world.remove_fluid(handle);
    }
}

//for now, just assume that everything is run in bevy's fixed step
pub fn step_simulation(
    mut salva_ctx: ResMut<SalvaContext>,
    time: Res<Time>
) {
    println!("test");
    salva_ctx.liquid_world.step(time.delta_secs(), &Vector::new(0., -9.81, 0.));
}
