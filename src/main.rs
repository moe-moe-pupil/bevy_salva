use bevy::{app::{App, Startup}, prelude::{Bundle, Camera3dBundle, Commands, Transform}, DefaultPlugins};
use bevy_flycam::{FlyCam, NoCameraPlayerPlugin};
use bevy_rapier3d::{plugin::{systems::init_colliders, RapierPhysicsPlugin}, prelude::{Collider, RigidBody}, render::RapierDebugRenderPlugin};
use plugin::SalvaPhysicsPlugin;
use salva3d::solver::DFSPHSolver;
use utils::cube_fluid;

mod plugin;
mod fluid;
mod utils;
mod systems;

pub const DEFAULT_PARTICLE_RADIUS: salva3d::math::Real = 0.05;
pub const DEFAULT_SMOOTHING_FACTOR: salva3d::math::Real = 2.0;

fn main() {
    let mut app = App::new();

    let fluid_solver: DFSPHSolver = DFSPHSolver::new();
    app.add_plugins((
        DefaultPlugins,
        RapierPhysicsPlugin::<()>::default(),
        RapierDebugRenderPlugin::default(),
        SalvaPhysicsPlugin::new(fluid_solver),
        NoCameraPlayerPlugin
    ));
    

    app.add_systems(Startup, startup);
    
    let fluid = cube_fluid(10, 10, 10, DEFAULT_PARTICLE_RADIUS, 1000.);
    app.run();
}

fn startup(
    mut commands: Commands
) {
    commands.spawn((
        Camera3dBundle::default(),
        FlyCam
    ));

    commands.spawn((
        RigidBody::Fixed,
        Collider::cuboid(10., 0.1, 10.),
        Transform::from_xyz(0., -0.1, 0.)
    ));
}
