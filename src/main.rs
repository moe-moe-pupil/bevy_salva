use bevy::{app::{App, Startup}, prelude::{Camera3dBundle, Commands, Transform}, DefaultPlugins};
use bevy_flycam::{FlyCam, NoCameraPlayerPlugin};
use bevy_rapier3d::{plugin::RapierPhysicsPlugin, prelude::{Collider, RigidBody}, render::RapierDebugRenderPlugin};
use plugin::SalvaPhysicsPlugin;
use salva3d::solver::DFSPHSolver;

mod plugin;

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
