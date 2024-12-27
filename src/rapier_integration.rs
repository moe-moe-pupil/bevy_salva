use bevy::prelude::Component;
use salva3d::object::BoundaryHandle;

pub enum ColliderSamplingMethod {
    StaticSampling,
    DynamicContactSampling,
}

impl Default for ColliderSamplingMethod {
    fn default() -> Self {
        //for some reason this method is faster,
        //so this should be the default.
        Self::DynamicContactSampling
    }
}

#[derive(Component, Default)]
pub struct RapierColliderSampling {
    pub sampling_method: ColliderSamplingMethod,
}



#[derive(Component)]
pub struct ColliderBoundaryHandle(pub BoundaryHandle);
