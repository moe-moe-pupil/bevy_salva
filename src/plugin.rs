use std::{mem, rc::Rc, sync::Arc};

use bevy::{app::{Plugin, PostUpdate}, ecs::{intern::Interned, schedule::{ScheduleLabel, SystemConfigs}}, prelude::Resource};
use bevy_rapier3d::plugin::PhysicsSet;
use salva3d::{math::Real, solver::{DFSPHSolver, PressureSolver}, LiquidWorld};

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

    // pub fn get_systems(set: PhysicsSet) -> SystemConfigs {
    //     match set {
    //         PhysicsSet::SyncBackend => (

    //         ).chain().into_configs(),
    //         PhysicsSet::StepSimulation => (

    //         ).chain().into_configs(),
    //         PhysicsSet::Writeback => (

    //         ).chain().into_configs()
    //     }
    // }
}

#[derive(Resource)]
pub struct SalvaContext {
    pub liquid_world: LiquidWorld
}

impl<S: PressureSolver + Send + Sync + 'static> Plugin for SalvaPhysicsPlugin<S> {
    fn build(&self, app: &mut bevy::prelude::App) {
        // SAFETY: this is fine because self.solver is private, meaning that 
        //         self.solver cannot be accessed after the app closes
        let solver: S = unsafe { std::mem::transmute_copy(&self.solver)};

        if self.default_rapier_coupling_config {
            app.insert_resource(SalvaContext {
                liquid_world: LiquidWorld::new(solver, self.particle_radius, self.smoothing_factor)
            });

            // TODO: add systems in system sets using self.get_systems()
        }
    }
}
