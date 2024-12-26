use bevy::{
    app::{App, Startup},
    input::ButtonInput,
    prelude::{Camera3dBundle, Commands, Entity, KeyCode, Query, Res, ResMut, Transform, With},
    time::{Fixed, Time},
    DefaultPlugins,
};
use bevy::prelude::FixedUpdate;
use bevy_flycam::{FlyCam, NoCameraPlayerPlugin};
use bevy_rapier3d::{
    na::Vector3,
    plugin::RapierPhysicsPlugin,
    prelude::{Collider, RigidBody},
    render::RapierDebugRenderPlugin,
};
use bevy_rapier3d::plugin::DefaultRapierContext;
use bevy_rapier3d::prelude::{RapierConfiguration, ReadDefaultRapierContext};
use fluid::{FluidParticlePositions, SalvaFluidHandle};
use plugin::{
    AppendNonPressureForces, RemoveNonPressureForcesAt, SalvaContext, SalvaPhysicsPlugin,
};
use salva3d::solver::{ArtificialViscosity, DFSPHSolver};
use utils::cube_particle_positions;

mod fluid;
mod plugin;
mod systems;
mod utils;

pub const DEFAULT_PARTICLE_RADIUS: salva3d::math::Real = 0.05;
pub const DEFAULT_SMOOTHING_FACTOR: salva3d::math::Real = 2.0;

fn main() {
    let mut app = App::new();

    app.insert_resource(Time::<Fixed>::from_hz(60.));

    let fluid_solver: DFSPHSolver = DFSPHSolver::new();
    app.add_plugins((
        DefaultPlugins,
        RapierPhysicsPlugin::<()>::default(),
        RapierDebugRenderPlugin::default(),
        SalvaPhysicsPlugin::new(fluid_solver)
            .in_schedule(FixedUpdate),
        NoCameraPlayerPlugin,
    ));


    app.add_systems(Startup, startup);

    app.run();
}

fn startup(mut commands: Commands, salva_ctx: Res<SalvaContext>) {
    commands.spawn((Camera3dBundle::default(), FlyCam));

    commands.spawn((
        RigidBody::Fixed,
        Collider::cuboid(10., 0.1, 10.),
        Transform::from_xyz(0., -0.1, 0.),
    ));

    //test fluid
    let fluid = commands
        .spawn((
            FluidParticlePositions {
                positions: cube_particle_positions(
                    10,
                    10,
                    10,
                    salva_ctx.liquid_world.particle_radius(),
                ),
            },
            AppendNonPressureForces(vec![Box::new(ArtificialViscosity::new(2.0, 0.0))]),
        ))
        .id();
}

pub fn update(
    mut commands: Commands,
    fluid_q: Query<Entity, With<SalvaFluidHandle>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::KeyG) {
        commands
            .get_entity(fluid_q.get_single().unwrap())
            .unwrap()
            .insert(RemoveNonPressureForcesAt(vec![0]));
    }
}

pub fn fixed_update(
    mut salva_ctx: ResMut<SalvaContext>,
    rapier_config: Query<&RapierConfiguration, With<DefaultRapierContext>>,
    time: Res<Time>,
) {
    let gravity = Vector3::from(rapier_config.get_single().unwrap().gravity);
    salva_ctx.liquid_world.step(time.delta_secs(), &gravity);
}
