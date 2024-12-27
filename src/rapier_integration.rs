use bevy::prelude::Component;
use salva3d::object::BoundaryHandle;

#[derive(Component)]
pub struct SampleRapierCollider;

#[derive(Component)]
pub struct ColliderBoundaryHandle(pub BoundaryHandle);
