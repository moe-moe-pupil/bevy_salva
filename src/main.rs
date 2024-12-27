use bevy::{
    app::{App, Startup},
    input::ButtonInput,
    prelude::{Camera3dBundle, Commands, Entity, KeyCode, Query, Res, ResMut, Transform, With},
    time::{Fixed, Time},
    DefaultPlugins,
};
use bevy::math::vec3;
use bevy::prelude::{Camera3d, Color, FixedUpdate, Gizmos, Isometry3d, Update, Vec3};
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
use crate::rapier_integration::SampleRapierCollider;

mod fluid;
mod plugin;
mod systems;
mod utils;
mod rapier_integration;

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


    app
        .add_systems(Startup, startup)
        .add_systems(Update, update);

    app.run();
}

fn startup(mut commands: Commands, salva_context: Res<SalvaContext>) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(5., 5., 5.)
            .looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
        FlyCam,
    ));

    commands.spawn((
        RigidBody::Fixed,
        Collider::cuboid(10., 0.1, 10.),
        Transform::from_xyz(0., -0.1, 0.),
        SampleRapierCollider
    ));

    //test fluid
    let mut positions = cube_particle_positions(
        10,
        10,
        10,
        salva_context.liquid_world.particle_radius(),
    );
    positions
        .iter_mut()
        .for_each(|p| *p += vec3(0., 5., 0.));
    let fluid = commands
        .spawn((
            FluidParticlePositions {
                positions,
            },
            AppendNonPressureForces(vec![Box::new(ArtificialViscosity::new(2.0, 0.0))]),
        ))
        .id();
}

pub fn update(
    mut commands: Commands,
    fluid_q: Query<(Entity, &FluidParticlePositions), With<SalvaFluidHandle>>,
    salva_context: Res<SalvaContext>,
    keys: Res<ButtonInput<KeyCode>>,
    mut gizmos: Gizmos
) {
    let result = fluid_q.get_single();
    if result.is_err() { return; }
    let (fluid_entity, positions) = result.unwrap();

    //draw particles
    for pos in positions.positions.iter() {
        gizmos.sphere(
            Isometry3d::from_xyz(pos.x, pos.y, pos.z),
            salva_context.liquid_world.particle_radius(),
            Color::linear_rgb(1., 0., 0.)
        );
    }

    //nonpressure force testing
    if keys.just_pressed(KeyCode::KeyG) {
        commands
            .get_entity(fluid_entity)
            .unwrap()
            .insert(RemoveNonPressureForcesAt(vec![0]));
    }
}

pub fn fixed_update(
    mut salva_context: ResMut<SalvaContext>,
    rapier_config: Query<&RapierConfiguration, With<DefaultRapierContext>>,
    time: Res<Time>,
) {
    let gravity = Vector3::from(rapier_config.get_single().unwrap().gravity);
    salva_context.liquid_world.step(time.delta_secs(), &gravity);
}
