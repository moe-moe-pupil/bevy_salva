use std::collections::HashMap;

use crate::plugin::systems;
use bevy::prelude::{IntoSystemSetConfigs, SystemSet, TransformSystem};
use bevy::{
    app::{Plugin, PostUpdate},
    ecs::{
        intern::Interned,
        schedule::{ScheduleLabel, SystemConfigs},
    },
    prelude::{Entity, IntoSystemConfigs, Resource},
};
use salva::{
    object::FluidHandle,
    solver::PressureSolver,
    LiquidWorld,
};
use salva::math::Vector;
use crate::math::Real;

#[cfg(feature = "rapier")]
use crate::rapier_integration;
#[cfg(feature = "rapier")]
use salva::integrations::rapier::ColliderCouplingSet;
#[cfg(feature = "rapier")]
use bevy_rapier::plugin::PhysicsSet;
#[cfg(feature = "rapier")]
use bevy_rapier::prelude::RapierContext;

//TODO: use a feature for enabling coupling with bevy_rapier
pub struct SalvaPhysicsPlugin<S: PressureSolver + Send + Sync + 'static> {
    schedule: Interned<dyn ScheduleLabel>,
    default_rapier_coupling_config: bool,
    solver: S,
    particle_radius: Real,
    smoothing_factor: Real,
}

impl<S: PressureSolver + Send + Sync + 'static> SalvaPhysicsPlugin<S> {
    pub const DEFAULT_PARTICLE_RADIUS: Real = 0.05;
    pub const DEFAULT_SMOOTHING_FACTOR: Real = 2.0;

    pub fn new(solver: S) -> Self {
        Self {
            schedule: PostUpdate.intern(),
            default_rapier_coupling_config: true,
            solver,
            particle_radius: Self::DEFAULT_PARTICLE_RADIUS,
            smoothing_factor: Self::DEFAULT_SMOOTHING_FACTOR,
        }
    }

    pub fn in_schedule(mut self, schedule: impl ScheduleLabel) -> Self {
        self.schedule = schedule.intern();
        self
    }

    pub fn with_solver(mut self, solver: S) -> Self {
        self.solver = solver;
        self
    }

    pub fn with_particle_radius(mut self, particle_radius: Real) -> Self {
        self.particle_radius = particle_radius;
        self
    }

    pub fn with_smoothing_factor(mut self, smoothing_factor: Real) -> Self {
        self.smoothing_factor = smoothing_factor;
        self
    }

    pub fn use_default_rapier_coupling(mut self, use_default_coupling: bool) -> Self {
        self.default_rapier_coupling_config = use_default_coupling;
        self
    }

    pub fn get_systems(set: SalvaSimulationSet) -> SystemConfigs {
        #[cfg(feature = "rapier")]
        match set {
            SalvaSimulationSet::SyncBackend => (
                systems::sync_removals,
                systems::init_fluids,
                systems::apply_nonpressure_force_changes,
                rapier_integration::sample_rapier_colliders,
            )
                .chain()
                .in_set(SalvaSimulationSet::SyncBackend),
            SalvaSimulationSet::StepSimulation => {
                (systems::step_simulation).in_set(SalvaSimulationSet::StepSimulation)
            }
            SalvaSimulationSet::Writeback => (systems::writeback_particle_positions,)
                .chain()
                .in_set(SalvaSimulationSet::Writeback),
        }

        #[cfg(not(feature = "rapier"))]
        match set {
            SalvaSimulationSet::SyncBackend => (
                systems::sync_removals,
                systems::init_fluids,
                systems::apply_nonpressure_force_changes,
            )
                .chain()
                .in_set(SalvaSimulationSet::SyncBackend),
            SalvaSimulationSet::StepSimulation => {
                (systems::step_simulation).in_set(SalvaSimulationSet::StepSimulation)
            }
            SalvaSimulationSet::Writeback => (systems::writeback_particle_positions,)
                .chain()
                .in_set(SalvaSimulationSet::Writeback),
        }
    }
}

impl<S: PressureSolver + Send + Sync + 'static> Plugin for SalvaPhysicsPlugin<S> {
    fn build(&self, app: &mut bevy::prelude::App) {
        // SAFETY: this is fine because self.solver is private, meaning that
        //         self.solver cannot be accessed after the app closes
        let solver: S = unsafe { std::mem::transmute_copy(&self.solver) };

        app.insert_resource(SalvaContext {
            liquid_world: LiquidWorld::new(solver, self.particle_radius, self.smoothing_factor),
            entity2fluid: HashMap::default(),
            #[cfg(feature = "rapier")]
            coupling: ColliderCouplingSet::new(),
        });

        if self.schedule != PostUpdate.intern() {
            app.add_systems(
                PostUpdate,
                (systems::sync_removals,).before(TransformSystem::TransformPropagate),
            );
        }

        #[cfg(feature = "rapier")]
        if self.default_rapier_coupling_config {
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

            app.add_systems(
                self.schedule,
                (
                    Self::get_systems(SalvaSimulationSet::SyncBackend),
                    Self::get_systems(SalvaSimulationSet::StepSimulation),
                    Self::get_systems(SalvaSimulationSet::Writeback),
                ),
            );

            //TODO: implement a TimestepMode like how bevy_rapier has it
        }
    }
}

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

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum SalvaSimulationSet {
    SyncBackend,
    StepSimulation,
    Writeback,
}
