use bevy::prelude::Component;
use bevy_rapier3d::parry::math::Point;
use salva3d::object::BoundaryHandle;

#[allow(unused_imports)]
use crate::plugin::SalvaPhysicsPlugin;

pub enum ColliderSamplingMethod {
    /// Collider shape is approximated for the fluid simulation in a way that keeps its shape consistent.
    /// The shape is determined using [`salva3d::sampling::shape_surface_ray_sample`]
    ///
    /// Good for smaller objects with finer details. Larger objects cause performance issues.
    Static,
    /// Collider shape is approximated on-the-fly as fluid particles make contact with it.
    ///
    /// Performance is more consistent for shapes of any size at the cost of less detail.
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

#[derive(Component, Default)]
pub struct RapierColliderSampling {
    pub sampling_method: ColliderSamplingMethod,
}

#[derive(Component)]
pub struct ColliderBoundaryHandle(pub BoundaryHandle);
