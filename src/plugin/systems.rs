use crate::fluid::{FluidAccelerations, FluidDensity, FluidInteractionGroups, FluidNonPressureForces, FluidPositions, FluidVelocities, SalvaFluidHandle};
use bevy::prelude::{error, Changed, Commands, Entity, Query, RemovedComponents, Res, Time, With, Without};
use bevy_rapier::prelude::RapierConfiguration;
use salva::object::interaction_groups::InteractionGroups;
use salva::{math::Point, object::Fluid};
use salva::math::Vector;
use crate::fluid::{AppendNonPressureForces, RemoveNonPressureForcesAt};
use crate::math::Vect;
use crate::plugin::salva_context::SalvaContext;
use crate::plugin::{DefaultSalvaContext, SalvaConfiguration, SalvaContextAccess, SalvaContextEntityLink, SimulationToRenderTime, TimestepMode, WriteSalvaContext};
use crate::rapier_integration::SalvaRapierCoupling;
use crate::utils;

pub fn init_fluids(
    mut commands: Commands,
    mut new_fluids: Query<
        (
            Entity,
            Option<&SalvaContextEntityLink>,
            &FluidPositions,
            Option<&FluidDensity>,
            Option<&mut FluidNonPressureForces>,
            Option<&FluidInteractionGroups>,
        ),
        Without<SalvaFluidHandle>,
    >,
    q_default_context: Query<Entity, With<DefaultSalvaContext>>,
    mut q_contexts: Query<&mut SalvaContext>,
) {
    for (
        entity,
        context_link,
        particle_positions,
        density,
        nonpressure_forces,
        fluid_interaction_groups,
    ) in new_fluids.iter_mut() {
        let mut entity_cmd = commands.entity(entity);

        let density = density.map_or_else(|| 1000.0, |d| d.density0);

        let particle_positions: Vec<_> = particle_positions
            .iter()
            .map(|v| Point::from(*v))
            .collect();

        let context_entity = context_link.map_or_else(
            || {
                let context_entity = q_default_context.get_single().ok()?;
                entity_cmd.insert(SalvaContextEntityLink(context_entity));
                Some(context_entity)
            },
            |link| Some(link.0)
        );

        let Some(context_entity) = context_entity else {
            continue;
        };

        let Ok(mut context) = q_contexts.get_mut(context_entity) else {
            error!("Couldn't find salva context entity {context_entity} while initializing {entity}");
            continue;
        };

        let mut salva_fluid = Fluid::new(
            particle_positions,
            context.liquid_world.particle_radius(),
            density,
            fluid_interaction_groups.map_or_else(
                InteractionGroups::default,
                |groups| (*groups).into()
            )
        );
        if let Some(mut nonpressure_forces) = nonpressure_forces {
            salva_fluid
                .nonpressure_forces
                .append(&mut nonpressure_forces.0);
        }
        let fluid_handle = context.liquid_world.add_fluid(salva_fluid);
        entity_cmd.insert(SalvaFluidHandle(fluid_handle));
        context.entity2fluid.insert(entity, fluid_handle);
    }
}

pub fn apply_fluid_user_changes(
    mut context_writer: WriteSalvaContext,
    mut append_q: Query<
        (&SalvaFluidHandle, &SalvaContextEntityLink, &mut AppendNonPressureForces),
        Changed<AppendNonPressureForces>,
    >,
    mut remove_at_q: Query<
        (&SalvaFluidHandle, &SalvaContextEntityLink, &mut RemoveNonPressureForcesAt),
        Changed<RemoveNonPressureForcesAt>,
    >,
    changed_positions: Query<
        (&SalvaFluidHandle, &SalvaContextEntityLink, &FluidPositions),
        Changed<FluidPositions>
    >,
    changed_velocities: Query<
        (&SalvaFluidHandle, &SalvaContextEntityLink, &FluidPositions, &FluidVelocities),
        Changed<FluidVelocities>
    >,
    changed_accelerations: Query<
        (&SalvaFluidHandle, &SalvaContextEntityLink, &FluidPositions, &FluidAccelerations),
        Changed<FluidAccelerations>
    >,
) {
    // Handles nonpressure forces the user wants to append to fluids
    for (handle, link, mut appends) in append_q.iter_mut() {
        let mut context = context_writer.context(link);
        context
            .liquid_world
            .fluids_mut()
            .get_mut(handle.0)
            .unwrap()
            .nonpressure_forces
            .append(&mut appends.0);
    }

    // Handles nonpressure forces the user wants to remove from fluids
    for (handle, link, mut removals) in remove_at_q.iter_mut() {
        let mut context = context_writer.context(link);
        let nonpressure_forces = &mut context
            .liquid_world
            .fluids_mut()
            .get_mut(handle.0)
            .unwrap()
            .nonpressure_forces;

        for i in removals.0.iter() { nonpressure_forces.remove(*i); }
        removals.0.clear();
    }

    for (handle, link, positions) in changed_positions.iter() {
        let mut context = context_writer.context(link);
        let radius = context.liquid_world.particle_radius();
        let mut fluid = context.liquid_world
            .fluids_mut()
            .get_mut(handle.0)
            .unwrap();
        // Set positions
        fluid.positions = positions
                .iter()
                .copied()
                .map(|v|  Point::from(v))
                .collect();
        // Reset velocities & accelerations
        // TODO: allow deleting individual particles without having to wipe all kinematic data
        fluid.velocities = std::iter::repeat(Vector::zeros())
            .take(positions.len())
            .collect();
        fluid.accelerations = std::iter::repeat(Vector::zeros())
            .take(positions.len())
            .collect();
        // Set volumes
        let volume = utils::particle_volume(radius);
        fluid.volumes = std::iter::repeat(volume)
            .take(positions.len())
            .collect();
    }

    for (handle, link, positions, vels) in changed_velocities.iter() {
        let mut context = context_writer.context(link);
        let vels =
            if vels.len() == positions.len() {
                vels
                    .iter()
                    .copied()
                    .map(|v| Vector::from(v))
                    .collect()
            }
            else {
                std::iter::repeat(Vector::zeros())
                    .take(positions.len())
                    .collect()
            };
        context.liquid_world
            .fluids_mut()
            .get_mut(handle.0)
            .unwrap()
            .velocities = vels;
    }

    for (handle, link, positions, accs) in changed_accelerations.iter() {
        let mut context = context_writer.context(link);
        let accs =
            if accs.len() == positions.len() {
                accs
                    .iter()
                    .copied()
                    .map(|v| Vector::from(v))
                    .collect()
            }
            else {
                std::iter::repeat(Vector::zeros())
                    .take(positions.len())
                    .collect()
            };
        context.liquid_world
            .fluids_mut()
            .get_mut(handle.0)
            .unwrap()
            .accelerations = accs;
    }
}

pub fn sync_removals(
    mut removed_particle_positions: RemovedComponents<FluidPositions>,
    mut removed_fluids: RemovedComponents<SalvaFluidHandle>,
    mut context_writer: WriteSalvaContext
) {
    //remove fluids whos entities had their salva fluid handle or fluid particle components removed
    for entity in removed_fluids
        .read()
        .chain(removed_particle_positions.read())
    {
        if let Some((mut context, handle)) = context_writer.salva_context
            .iter_mut()
            .find_map(|mut context| {
                context.entity2fluid.remove(&entity).map(|h| (context, h))
            })
        {
            context.liquid_world.remove_fluid(handle);
        }
    }
}

/// The system that steps [`SalvaContext`]s that run independently.
/// See `SalvaConfiguration.physics_pipeline_active` for more details.
pub fn step_simulation(
    mut salva_context: Query<(&mut SalvaContext, &SalvaConfiguration, &mut SimulationToRenderTime)>,
    timestep_mode: Res<TimestepMode>,
    time: Res<Time>,
) {
    for (mut context, config, mut sim_to_render_time) in salva_context.iter_mut() {
        // If this context runs independently and its physics pipeline is active,
        // step its simulation.
        if config.physics_pipeline_active.is_some_and(|active| active) {
            context.step_simulation(
                &time,
                &config.gravity.into(),
                timestep_mode.clone(),
                &mut sim_to_render_time
            );
        }
    }
}

/// Write back fluid particle positions, velocities, and accelerations.
pub fn writeback_particle_kinematics(
    read_context: SalvaContextAccess,
    salva_configs: Query<&SalvaConfiguration>,
    #[cfg(feature = "rapier")]
    rapier_configs: Query<&RapierConfiguration>,
    #[cfg(feature = "rapier")]
    rapier_couplings: Query<&SalvaRapierCoupling>,
    mut fluid_pos_q: Query<(
        &SalvaFluidHandle,
        &SalvaContextEntityLink,
        &mut FluidPositions,
        &mut FluidVelocities,
        &mut FluidAccelerations,
    )>,
) {
    for (
        handle,
        link,
        mut positions,
        mut vels,
        mut accs,
    ) in fluid_pos_q.iter_mut() {
        let config = salva_configs.get(link.0).unwrap();

        #[cfg(not(feature = "rapier"))]
        let should_writeback = config.physics_is_independently_active();

        // Check if salva ctx is not independent is coupled to Rapier, and Rapier physics is active
        #[cfg(feature = "rapier")]
        let rapier_coupling = rapier_couplings.get(link.0);
        #[cfg(feature = "rapier")]
        let should_writeback = config.physics_pipeline_active.is_none() &&
            rapier_coupling.is_ok() &&
            rapier_configs
                .get(rapier_coupling.unwrap().rapier_context_entity)
                .unwrap()
                .physics_pipeline_active;

        if should_writeback {
            let context = read_context.context(link);
            let fluid = &context.liquid_world.fluids().get(handle.0).unwrap();
            **positions = fluid.positions
                .iter()
                .map(|v| Vect::from(*v))
                .collect();
            **vels = fluid.velocities
                .iter()
                .map(|v| Vect::from(*v))
                .collect();
            **accs = fluid.accelerations
                .iter()
                .map(|v| Vect::from(*v))
                .collect();
        }
    }
}

