use std::collections::HashMap;

use bevy::asset::{AssetPlugin, AssetServer, Assets, Handle, LoadState};
use bevy::core_pipeline::core_2d::Camera2d;
use bevy::ecs::component::Component;
use bevy::ecs::event::EventReader;
use bevy::ecs::name::Name;
use bevy::ecs::resource::Resource;
use bevy::ecs::system::{ResMut, Single};
use bevy::image::Image;
use bevy::input::mouse::MouseWheel;
use bevy::math::Vec2;
use bevy::prelude::{Color, FixedUpdate, Gizmos, Update, Vec3};
use bevy::render::texture::ImagePlugin;
use bevy::sprite::Sprite;
use bevy::state::app::AppExtStates;
use bevy::state::condition::in_state;
use bevy::state::state::{NextState, OnExit, States};
use bevy::window::{PrimaryWindow, Window, WindowPlugin};
use bevy::{
    app::{App, Startup},
    input::ButtonInput,
    prelude::*,
    time::{Fixed, Time},
};
use bevy_collider_gen::prelude::edges::Edges;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_prototype_lyon::{prelude::*, shapes};
use bevy_rapier2d::plugin::DefaultRapierContext;
use bevy_rapier2d::prelude::{
    Ccd, ColliderMassProperties, MassProperties, RapierConfiguration, TriMeshFlags, Velocity,
};
use bevy_rapier2d::{
    plugin::RapierPhysicsPlugin,
    prelude::{Collider, RigidBody},
    render::RapierDebugRenderPlugin,
};
use bevy_salva2d::fluid::{FluidPositions, SalvaFluidHandle};
use bevy_salva2d::plugin::{
    AppendNonPressureForces, RemoveNonPressureForcesAt, SalvaContext, SalvaContextInitialization,
    SalvaPhysicsPlugin,
};
use bevy_salva2d::rapier_integration::RapierColliderSampling;
use bevy_salva2d::salva::{math::Real, solver::ArtificialViscosity};
use nalgebra::Vector2;

fn main() {
    let mut app = App::new();

    app.insert_resource(Time::<Fixed>::from_hz(60.));
    app.register_type::<ColliderMassProperties>();
    app.add_plugins((
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "colliders".to_string(),
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                file_path: ".".to_string(),
                ..default()
            }),
        ShapePlugin,
        RapierPhysicsPlugin::<()>::default(),
        RapierDebugRenderPlugin::default(),
        EguiPlugin {
            enable_multipass_for_primary_context: true,
        },
        WorldInspectorPlugin::new(), // Add the inspector plugin
        SalvaPhysicsPlugin::new()
            .with_custom_world_initialization(
                SalvaContextInitialization::InitializeDefaultSalvaContext {
                    particle_radius: 0.5,
                    smoothing_factor: 2.0,
                },
            )
            .in_schedule(FixedUpdate),
    ));
    app.init_state::<AppState>()
        .insert_resource(GameAsset::default())
        .add_systems(Startup, load_assets);
    // .add_systems(OnExit(AppState::Loading), (ship_spawn,));

    app.add_systems(
        FixedUpdate,
        (
            check_assets.run_if(in_state(AppState::Loading)),
            ship_movement.run_if(in_state(AppState::Running)),
        ),
    );

    app.add_systems(OnExit(AppState::Loading), startup)
        .add_systems(FixedUpdate, (update, debug_camera));

    app.run();
}

fn ship_spawn(
    mut commands: Commands,
    game_assets: Res<GameAsset>,
    image_assets: Res<Assets<Image>>,
) {
    let Some(sprite_handle) = game_assets.image_handles.get("ship") else {
        return;
    };
    let sprite_image = image_assets.get(sprite_handle).unwrap();
    let edges = Edges::try_from(sprite_image).unwrap();
    let edge_coordinate_groups = edges.multi_translated();
    for group in edge_coordinate_groups {
        let coords: Vec<_> = group.into_iter().map(|v| v / 1.0).collect();
        let indices: Vec<[u32; 3]> = (0..coords.len())
            .map(|i| {
                [
                    i as u32,
                    (i + 1) as u32 % (coords.len() as u32),
                    (i) as u32 % (coords.len() as u32),
                ]
            })
            .collect();
        let collider = Collider::trimesh_with_flags(
            coords,
            indices,
            TriMeshFlags::HALF_EDGE_TOPOLOGY | TriMeshFlags::FIX_INTERNAL_EDGES,
        )
        .unwrap();
        commands.spawn((
            Ship,
            collider,
            // ColliderMassProperties::MassProperties(MassProperties {
            //     local_center_of_mass: Vec2::new(0.0, -1.0),
            //     mass: 1000.0,
            //     principal_inertia: 10.0,
            // }),
            Ccd::enabled(),
            RigidBody::Dynamic,
            Transform::from_xyz(0.0, 100.0, 0.0),
            RapierColliderSampling::default(),
            Sprite {
                image: sprite_handle.clone(),
                ..default()
            },
        ));
    }
}

pub fn check_assets(
    asset_server: Res<AssetServer>,
    game_assets: Res<GameAsset>,
    mut state: ResMut<NextState<AppState>>,
) {
    let all_images_loaded = game_assets.image_handles.values().all(|handle| {
        asset_server
            .get_load_state(handle)
            .is_some_and(|state| matches!(state, LoadState::Loaded))
    });
    if all_images_loaded {
        state.set(AppState::Running);
    }
}

pub fn load_assets(asset_server: Res<AssetServer>, mut game_assets: ResMut<GameAsset>) {
    game_assets.image_handles = HashMap::from([("ship", asset_server.load("assets/ship.png"))]);
}

#[derive(Component)]
#[require(RigidBody,Velocity, Transform = INITIAL_POSITION)]
pub struct Ship;

pub fn ship_movement(
    mut query: Query<(&mut Transform, &mut Velocity), With<Ship>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    for (mut transform, mut velocity) in &mut query {
        let linear_velocity = &mut velocity.linvel;
        for key in keys.get_pressed() {
            match key {
                KeyCode::KeyA => linear_velocity.x -= 3.,
                KeyCode::KeyD => linear_velocity.x += 3.,
                KeyCode::Digit1 => {
                    *linear_velocity = Vec2::default();
                    *transform = INITIAL_POSITION;
                }
                _ => {}
            }
        }
    }
}
const INITIAL_POSITION: Transform = Transform::from_xyz(0., 20., 0.);

///
/// After this, things don't differ in a way related to this crate, it's just some of my
/// personal boilerplate
///
#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
pub enum AppState {
    #[default]
    Loading,
    Running,
}

#[derive(Resource, Default)]
pub struct GameAsset {
    pub image_handles: HashMap<&'static str, Handle<Image>>,
}

fn debug_camera(
    keyboard_btns: Res<ButtonInput<KeyCode>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    mut query: Query<&mut Transform, With<Camera2d>>,
    mut scroll_evr: EventReader<MouseWheel>,
) {
    if let Some(position) = q_windows.single().unwrap().cursor_position() {
        for ev in scroll_evr.read() {
            for mut transform in query.iter_mut() {
                transform.scale -= 0.01 * ev.y;
            }
        }
        if keyboard_btns.pressed(KeyCode::ArrowDown) {
            for mut transform in query.iter_mut() {
                transform.translation.y -= 0.5;
            }
        }
        if keyboard_btns.pressed(KeyCode::ArrowUp) {
            for mut transform in query.iter_mut() {
                transform.translation.y += 0.5;
            }
        }
        if keyboard_btns.pressed(KeyCode::ArrowLeft) {
            for mut transform in query.iter_mut() {
                transform.translation.x -= 0.5;
            }
        }
        if keyboard_btns.pressed(KeyCode::ArrowRight) {
            for mut transform in query.iter_mut() {
                transform.translation.x += 0.5;
            }
        }
    }
}

fn startup(mut commands: Commands, game_assets: Res<GameAsset>, image_assets: Res<Assets<Image>>) {
    commands.spawn((
        Camera2d::default(),
        Transform::from_xyz(0., 0., 0.).with_scale(Vec3::new(0.05, 0.05, 1.0)),
    ));

    let water_container_half_size = Vec2::new(40.0, 40.0);
    let water_container_translation = Vec2::new(0.0, -20.0);
    commands.spawn((
        RigidBody::Fixed,
        Collider::cuboid(water_container_half_size.x, 1.0),
        Transform::from_xyz(0., water_container_translation.y * 2.0, 0.),
        RapierColliderSampling::default(),
    ));

    commands.spawn((
        RigidBody::Fixed,
        Collider::cuboid(1.0, water_container_half_size.y),
        Transform::from_xyz(
            water_container_translation.x + water_container_half_size.x,
            water_container_translation.y,
            0.,
        ),
        RapierColliderSampling::default(),
    ));

    commands.spawn((
        RigidBody::Fixed,
        Collider::cuboid(1.0, water_container_half_size.y),
        Transform::from_xyz(
            water_container_translation.x - water_container_half_size.x,
            water_container_translation.y,
            0.,
        ),
        RapierColliderSampling::default(),
    ));

    commands.spawn((
        RigidBody::Dynamic,
        Name::new("test"),
        Collider::cuboid(4., 4.),
        ColliderMassProperties::Mass(12000.0),
        Transform::from_xyz(0., 50.1, 0.),
        RapierColliderSampling::default(),
    ));

    let Some(sprite_handle) = game_assets.image_handles.get("ship") else {
        return;
    };
    let sprite_image = image_assets.get(sprite_handle).unwrap();
    let edges = Edges::try_from(sprite_image).unwrap();
    let edge_coordinate_groups = edges.multi_translated();
    for group in edge_coordinate_groups {
        let coords: Vec<_> = group.into_iter().map(|v| v / 1.0).collect();
        let indices: Vec<[u32; 3]> = (0..coords.len())
            .map(|i| {
                [
                    i as u32,
                    (i + 1) as u32 % (coords.len() as u32),
                    (i) as u32 % (coords.len() as u32),
                ]
            })
            .collect();
        let collider = Collider::trimesh_with_flags(
            coords,
            indices,
            TriMeshFlags::HALF_EDGE_TOPOLOGY | TriMeshFlags::FIX_INTERNAL_EDGES,
        )
        .unwrap();
        commands.spawn((
            Ship,
            Collider::ball(0.5),
            ColliderMassProperties::MassProperties(MassProperties {
                local_center_of_mass: Vec2::new(0.0, -1.0),
                mass: 1000.0,
                principal_inertia: 0.0,
            }),
            Ccd::enabled(),
            Transform::from_xyz(0.0, 120.0, 0.0),
            RapierColliderSampling::default(),
            Sprite {
                image: sprite_handle.clone(),
                ..default()
            },
        ));
    }

    // test fluid
    let mut positions = vec![Vec2 { x: 0.0, y: 0.0 }];

    for x in -20..=20 {
        for y in -40..=40 {
            positions.push(Vec2 {
                x: x as f32 * 2.0,
                y: y as f32 * 2.0,
            });
        }
    }

    positions.iter_mut().for_each(|p| *p += Vec2::new(0.0, 5.0));
    let _fluid = commands
        .spawn((
            FluidPositions(positions),
            AppendNonPressureForces(vec![Box::new(ArtificialViscosity::new(0.0, 0.0))]),
        ))
        .id();
}

pub fn update(
    mut commands: Commands,
    fluid_q: Query<(Entity, &FluidPositions), With<SalvaFluidHandle>>,
    salva_context: Query<&SalvaContext>,
    keys: Res<ButtonInput<KeyCode>>,
    mut gizmos: Gizmos,
) {
    let result = fluid_q.get_single();
    if result.is_err() {
        return;
    }
    let (fluid_entity, positions) = result.unwrap();

    //draw particles
    // Get the SalvaContext resource
    let salva_context = match salva_context.get_single() {
        Ok(ctx) => ctx,
        Err(_) => return,
    };

    // Draw particles
    for pos in positions.0.iter() {
        gizmos.circle_2d(
            Vec2::new(pos.x, pos.y),
            salva_context.liquid_world.particle_radius(),
            Color::linear_rgb(0.1843, 0.5647, 0.7686),
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
    mut salva_context: Single<&mut SalvaContext>,
    rapier_config: Query<&RapierConfiguration, With<DefaultRapierContext>>,
    time: Res<Time>,
) {
    let gravity = Vector2::from(rapier_config.get_single().unwrap().gravity);
    salva_context.liquid_world.step(time.delta_secs(), &gravity);
}
