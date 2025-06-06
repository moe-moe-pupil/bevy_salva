use std::collections::HashMap;

use crate::math::Real;
use crate::plugin::salva_context::SalvaContext;
use crate::plugin::{systems, DefaultSalvaContext, SalvaContextEntityLink, TimestepMode};
#[cfg(feature = "rapier")]
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::schedule::ScheduleConfigs;
use bevy::ecs::system::ScheduleSystem;
use bevy::prelude::*;
use bevy::{
    app::{Plugin, PostUpdate},
    ecs::{
        intern::Interned,
        schedule::{ScheduleLabel},
    },
};
use salva::solver::DFSPHSolver;
use salva::LiquidWorld;

#[cfg(feature = "rapier")]
use crate::rapier_integration;
#[cfg(feature = "rapier")]
use bevy::app::PostStartup;
#[cfg(feature = "rapier")]
use bevy_rapier::plugin::PhysicsSet;
#[cfg(feature = "rapier")]
use bevy_rapier::prelude::RapierPhysicsPlugin;

use super::SalvaConfiguration;

pub struct SalvaPhysicsPlugin {
    schedule: Interned<dyn ScheduleLabel>,
    default_system_setup: bool,
    world_setup: SalvaContextInitialization,
}

impl SalvaPhysicsPlugin {
    pub const DEFAULT_PARTICLE_RADIUS: Real = 0.05;
    pub const DEFAULT_SMOOTHING_FACTOR: Real = 2.0;

    pub fn new() -> Self {
        Self {
            schedule: PostUpdate.intern(),
            default_system_setup: true,
            world_setup: SalvaContextInitialization::InitializeDefaultSalvaContext {
                particle_radius: Self::DEFAULT_PARTICLE_RADIUS,
                smoothing_factor: Self::DEFAULT_SMOOTHING_FACTOR
            }
        }
    }

    pub fn in_schedule(mut self, schedule: impl ScheduleLabel) -> Self {
        self.schedule = schedule.intern();
        self
    }

    pub fn with_custom_world_initialization(mut self, world_setup: SalvaContextInitialization) -> Self {
        self.world_setup = world_setup;
        self
    }

    pub fn with_default_system_setup(mut self, use_default_coupling: bool) -> Self {
        self.default_system_setup = use_default_coupling;
        self
    }

    pub fn get_systems(set: SalvaSimulationSet) ->  ScheduleConfigs<ScheduleSystem> {
        #[cfg(feature = "rapier")]
        match set {
            SalvaSimulationSet::SyncBackend => {
                (
                    systems::sync_removals,
                    systems::init_fluids,
                    systems::apply_fluid_user_changes,
                    rapier_integration::sample_rapier_colliders,
                )
                    .chain()
                    .in_set(SalvaSimulationSet::SyncBackend)
                    .after(PhysicsSet::SyncBackend)
            },
            SalvaSimulationSet::StepSimulation => {
                (
                    systems::step_simulation,
                    rapier_integration::step_simulation_rapier_coupling
                )
                    .in_set(SalvaSimulationSet::StepSimulation)
                    .after(PhysicsSet::StepSimulation)
            }
            SalvaSimulationSet::Writeback => {
                (systems::writeback_particle_kinematics,)
                    .chain()
                    .in_set(SalvaSimulationSet::Writeback)
                    .after(PhysicsSet::Writeback)
            },
        }

        #[cfg(not(feature = "rapier"))]
        match set {
            SalvaSimulationSet::SyncBackend => (
                systems::sync_removals,
                systems::init_fluids,
                systems::apply_fluid_user_changes,
            )
                .chain()
                .in_set(SalvaSimulationSet::SyncBackend),
            SalvaSimulationSet::StepSimulation => {
                (systems::step_simulation).in_set(SalvaSimulationSet::StepSimulation)
            }
            SalvaSimulationSet::Writeback => (systems::writeback_particle_kinematics,)
                .chain()
                .in_set(SalvaSimulationSet::Writeback),
        }
    }
}

impl Default for SalvaPhysicsPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for SalvaPhysicsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .register_type::<DefaultSalvaContext>()
            .register_type::<SalvaContextInitialization>()
            .register_type::<SalvaContextEntityLink>();
        
        app.init_resource::<TimestepMode>();

        let default_world_init = app.world().get_resource::<SalvaContextInitialization>();
        if let Some(world_init) = default_world_init {
            warn!("SalvaPhysicsPlugin added but a `SalvaContextInitialization` resource was already existing.\
            This might overwrite previous configuration made via `SalvaPhysicsPlugin::with_custom_initialization`\
            or `SalvaPhysicsPlugin::with_length_unit`.
            The following resource will be used: {:?}", world_init);
        } else {
            app.insert_resource(self.world_setup.clone());
        }

        app.add_systems(
            PreStartup,
            (insert_default_world,)
                .chain()
        );

        if self.schedule != PostUpdate.intern() {
            app.add_systems(
                PostUpdate,
                (systems::sync_removals,).before(TransformSystem::TransformPropagate),
            );
        }

        if self.default_system_setup {
            #[cfg(not(feature = "rapier"))]
            app.configure_sets(
                self.schedule,
                (
                    SalvaSimulationSet::SyncBackend,
                    SalvaSimulationSet::StepSimulation,
                    SalvaSimulationSet::Writeback,
                )
                    .chain()
                    .before(TransformSystem::TransformPropagate)
            );

            #[cfg(feature = "rapier")]
            {
                // Ensure that rapier physics is added.
                if !app.is_plugin_added::<RapierPhysicsPlugin<()>>() {
                    app.add_plugins(RapierPhysicsPlugin::<()>::default());
                }
                app.configure_sets(
                    self.schedule,
                    (
                        SalvaSimulationSet::SyncBackend,
                        SalvaSimulationSet::StepSimulation,
                        SalvaSimulationSet::Writeback,
                    )
                        .chain()
                        .before(TransformSystem::TransformPropagate)
                        .after(PhysicsSet::Writeback),
                );
            }

            app.add_systems(
                self.schedule,
                (
                    Self::get_systems(SalvaSimulationSet::SyncBackend),
                    Self::get_systems(SalvaSimulationSet::StepSimulation),
                    Self::get_systems(SalvaSimulationSet::Writeback),
                ),
            );

            // This system needs to run a bit later to ensure that the default RapierContext is created.
            // The system that initializes the default rapier context isn't public, so this is the workaround
            // for now.
            #[cfg(feature = "rapier")]
            app.add_systems(
                PostStartup,
                rapier_integration::link_default_contexts.before(SalvaSimulationSet::SyncBackend)
            );

            //TODO: implement a TimestepMode like how bevy_rapier has it
        }
    }
}

/// Specifies a default configuration for the default [`SalvaContext`]
///
/// Designed to be passed as parameter to [`SalvaPhysicsPlugin::with_custom_world_initialization`].
#[derive(Resource, Debug, Reflect, Clone)]
pub enum SalvaContextInitialization {
    /// [`SalvaPhysicsPlugin`] will not spawn any entity containing [`SalvaContext`] automatically.
    ///
    /// You are responsible for creating a [`SalvaContext`],
    /// before spawning any Salva entities (rigidbodies, colliders, joints).
    ///
    /// You might be interested in adding [`DefaultSalvaContext`] to the created world.
    NoAutomaticSalvaContext,
    /// [`SalvaPhysicsPlugin`] will spawn an entity containing a [`SalvaContext`]
    /// automatically during [`PreStartup`], with the [`DefaultSalvaContext`] marker component.
    ///
    InitializeDefaultSalvaContext {
        /// See [`LiquidWorld::new()`] for information on `particle_radius`
        particle_radius: salva::math::Real,
        /// See [`LiquidWorld::new()`] for information on `smoothing_factor`
        smoothing_factor: salva::math::Real,
    },
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum SalvaSimulationSet {
    SyncBackend,
    StepSimulation,
    Writeback,
}

pub fn insert_default_world(
    mut commands: Commands,
    initialization_data: Res<SalvaContextInitialization>,
) {
    match initialization_data.as_ref() {
        SalvaContextInitialization::NoAutomaticSalvaContext => {}
        SalvaContextInitialization::InitializeDefaultSalvaContext {
            particle_radius, smoothing_factor
        } => {
            let solver: DFSPHSolver = DFSPHSolver::new();
            // Required SalvaConfiguration is added automatically w/ default values
            commands.spawn((
                Name::new("Salva Context"),
                SalvaContext {
                    liquid_world: LiquidWorld::new(solver, *particle_radius, *smoothing_factor),
                    entity2fluid: HashMap::default(),
                },            
                #[cfg(feature = "rapier")]
                SalvaConfiguration {
                    physics_pipeline_active: None,
                    ..Default::default()
                },
                DefaultSalvaContext,
            ));
        }
    }
}
