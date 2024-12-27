use bevy::prelude::Component;
use salva3d::object::BoundaryHandle;

pub enum ColliderSamplingMethod {
    /// Collider shape is approximated for the fluid simulation in a way that keeps its shape consistent.
    ///
    /// Good for smaller objects with finer details. Larger objects cause performance issues.
    StaticSampling,
    /// Collider shape is approximated on-the-fly as fluid particles make contact with it.
    ///
    /// Performance is more consistent for shapes of any size at the cost of less detail.
    DynamicContactSampling,
}

impl Default for ColliderSamplingMethod {
    fn default() -> Self {
        Self::DynamicContactSampling
    }
}

#[derive(Component, Default)]
pub struct RapierColliderSampling {
    pub sampling_method: ColliderSamplingMethod,
}

#[derive(Component)]
pub struct ColliderBoundaryHandle(pub BoundaryHandle);
