#[allow(unused_imports)]
use crate::plugin::SalvaPhysicsPlugin;
use crate::plugin::{DefaultSalvaContext, SalvaConfiguration, SalvaContext, SalvaContextEntityLink, SalvaContextInitialization, SimulationToRenderTime, TimestepMode, WriteSalvaContext};
use bevy::prelude::{Commands, Component, Entity, Query, Res, Time, With, Without};
use bevy_rapier::geometry::RapierColliderHandle;
use bevy_rapier::parry::math::Point;
use bevy_rapier::plugin::{DefaultRapierContext, RapierConfiguration, WriteRapierContext};
use bevy_rapier::prelude::{CollisionGroups, RapierContextEntityLink};
use salva::integrations::rapier::{ColliderCouplingSet, ColliderSampling};
use salva::object::interaction_groups::InteractionGroups;
use salva::object::{Boundary, BoundaryHandle};

/// How a Rapier collider should be sampled for interaction with a Salva physics
/// world.
pub enum ColliderSamplingMethod {
    /// Collider shape is approximated for the fluid simulation in a way that keeps its shape consistent.
    /// The shape is determined using [`salva3d::sampling::shape_surface_ray_sample`]
    ///
    /// Good for smaller objects with finer details. Larger objects cause performance issues.
    Static,
    /// Collider shape is approximated on-the-fly as fluid particles make contact with it.
    ///
    /// Performance is more consistent for shapes of any size at the cost of accuracy.
    DynamicContact,
    /// Custom collider shape approximated with the given sample points in local-space.
    ///
    /// It is recommended that the points are separated by a distance smaller or equal to twice
    /// the particle radius used to initialize the fluid simulation world.
    /// The default particle radius is [`SalvaPhysicsPlugin::DEFAULT_PARTICLE_RADIUS`].
    CustomStatic(Vec<Point<f32>>),
}

impl Default for ColliderSamplingMethod {
    fn default() -> Self {
        Self::DynamicContact
    }
}

/// Add this to an entity with a [`Collider`](bevy_rapier::prelude::Collider)
/// to let it interact with a Salva physics world.
#[derive(Component, Default)]
pub struct RapierColliderSampling {
    pub sampling_method: ColliderSamplingMethod,
}

#[derive(Component)]
pub struct ColliderBoundaryHandle(pub BoundaryHandle);

/// The component added to [`SalvaContext`] entities that declares which [`RapierContext`]
/// entity a [`SalvaContext`] entity has its simulation coupled with.
///
/// Also contains the coupling manager ([`ColliderCouplingSet`]) needed for coupling rapier with salva.
#[derive(Component)]
pub struct SalvaRapierCoupling {
    /// The [`RapierContext`] entity that this [`SalvaContext`] entity is coupled with
    pub rapier_context_entity: Entity,
    /// The structure used in coupling rapier colliders with salva fluid boundaries to simulate.
    /// rigidbody-fluid interactions
    pub coupling: ColliderCouplingSet,
}

// WIP: for now, just assume that everything is run in bevy's fixed update step
pub fn step_simulation_rapier_coupling(
    mut salva_context_q: Query<(
        &mut SalvaContext,
        &mut SalvaRapierCoupling,
        &SalvaConfiguration,
        &mut SimulationToRenderTime
    )>,
    timestep_mode: Res<TimestepMode>,
    mut write_rapier_context: WriteRapierContext<()>,
    rapier_configs: Query<&RapierConfiguration>,
    time: Res<Time>,
) {
    for (
        mut context,
        mut link,
        config,
        mut sim_to_render_time
    ) in salva_context_q.iter_mut() {
        // Skip if this SalvaContext runs independently
        if config.physics_pipeline_active.is_some() {
            continue;
        }
        let (
            _,
            mut colliders,
            _,
            _,
            mut rigidbody_set,
        ) = write_rapier_context.rapier_context
            .get_mut(link.rapier_context_entity)
            .expect("Couldn't find RapierContext coupled to SalvaContext entity {entity}");

        let rapier_config = rapier_configs.get(link.rapier_context_entity)
            .expect("RapierContext entity doesn't have a RapierConfiguration!");
        if rapier_config.physics_pipeline_active {
            context.step_with_coupling(
                &time,
                &config.gravity.into(),
                timestep_mode.clone(),
                &mut sim_to_render_time,
                &mut link.coupling.as_manager_mut(&mut colliders.colliders, &mut rigidbody_set.bodies),
            );
        }
    }
}

/// The system responsible for sampling/coupling rapier colliders for rapier-salva coupling
/// by converting them into fluid boundaries.
pub fn sample_rapier_colliders(
    mut commands: Commands,
    colliders: Query<
        (
            Entity,
            &RapierContextEntityLink,
            Option<&SalvaContextEntityLink>,
            &RapierColliderHandle,
            &RapierColliderSampling,
            Option<&CollisionGroups>,
        ),
        Without<ColliderBoundaryHandle>,
    >,
    mut rapier_coupling_q: Query<&mut SalvaRapierCoupling>,
    q_default_context: Query<Entity, With<DefaultSalvaContext>>,
    mut context_writer: WriteSalvaContext,
    mut rapier_context_access: WriteRapierContext<()>,
) {
    for (
        entity,
        rapier_link,
        salva_link,
        co_handle,
        sampling,
        collision_groups,
    ) in colliders.iter() {
        let mut entity_cmd = commands.entity(entity);
        let salva_link = salva_link.map_or_else(
            || {
                let context_entity = q_default_context.get_single().unwrap();
                entity_cmd.insert(SalvaContextEntityLink(context_entity));
                SalvaContextEntityLink(context_entity)
            },
            |link| *link
        );

        let mut salva_context = context_writer.context(&salva_link);
        let radius = salva_context.liquid_world.particle_radius();
        let coupling = &mut rapier_coupling_q.get_mut(salva_link.0).unwrap().coupling;

        let (
            _,
            mut colliders,
            _,
            _,
            _,
        ) = rapier_context_access.rapier_context.get_mut(rapier_link.0)
            .unwrap();
        let colliders = &mut colliders.colliders;
        let co = colliders.get(co_handle.0).unwrap();

        let bo_handle = salva_context
            .liquid_world
            .add_boundary(Boundary::new(
                Vec::new(),
                collision_groups.map_or_else(
                    InteractionGroups::default,
                    |groups| InteractionGroups {
                        memberships: salva::object::interaction_groups::Group::from_bits(groups.memberships.bits())
                            .unwrap(),
                        filter: salva::object::interaction_groups::Group::from_bits(groups.filters.bits())
                            .unwrap(),
                    }
                )
            ));
        coupling.register_coupling(
            bo_handle,
            co_handle.0,
            match &sampling.sampling_method {
                ColliderSamplingMethod::Static => {
                    let samples =
                        salva::sampling::shape_surface_ray_sample(co.shape(), radius).unwrap();
                    ColliderSampling::StaticSampling(samples)
                }
                ColliderSamplingMethod::DynamicContact => {
                    ColliderSampling::DynamicContactSampling
                }
                ColliderSamplingMethod::CustomStatic(samples) => {
                    ColliderSampling::StaticSampling(samples.clone())
                }
            },
        );

        entity_cmd
            .insert(ColliderBoundaryHandle(bo_handle));
    }
}

/// System that links the default salva context to the default rapier context
#[cfg(feature = "rapier")]
pub fn link_default_contexts(
    mut commands: Commands,
    initialization_data: Res<SalvaContextInitialization>,
    mut default_salva_context: Query<
        (Entity, &mut SalvaConfiguration),
        (With<DefaultSalvaContext>, Without<SalvaRapierCoupling>)
    >,
    default_rapier_context: Query<(Entity, &RapierConfiguration), With<DefaultRapierContext>>,
) {
    match initialization_data.as_ref() {
        SalvaContextInitialization::NoAutomaticSalvaContext => {}
        SalvaContextInitialization::InitializeDefaultSalvaContext {
            particle_radius: _particle_radius, smoothing_factor: _smoothing_factor
        } => {
            let (salva_context_entity, mut salva_config) = default_salva_context
                .get_single_mut().unwrap();
            let (rapier_context_entity, rapier_config) = default_rapier_context
                .single();
            commands.entity(salva_context_entity)
                .insert(SalvaRapierCoupling {
                    rapier_context_entity,
                    coupling: ColliderCouplingSet::new(),
                });
            salva_config.gravity = rapier_config.gravity;
        }
    }
}
