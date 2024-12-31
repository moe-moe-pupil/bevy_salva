
pub extern crate nalgebra as na;
#[cfg(feature = "dim2")]
pub extern crate salva2d as salva;
#[cfg(feature = "dim3")]
pub extern crate salva3d as salva;

#[cfg(all(feature = "dim2", feature = "rapier"))]
pub extern crate bevy_rapier2d as bevy_rapier;
#[cfg(all(feature = "dim3", feature = "rapier"))]
pub extern crate bevy_rapier3d as bevy_rapier;

/// Math type aliases based on the dimension the engine is using
#[cfg(feature = "dim2")]
pub mod math {
    use bevy::math::Vec2;
    /// The real type (f32 or f64).
    pub type Real = salva::math::Real;
    /// The vector type.
    pub type Vect = Vec2;
    /// The rotation type (in 2D this is an angle in radians).
    pub type Rot = Real;
}

/// Type aliases to select the right vector/rotation types based
/// on the dimension used by the engine.
#[cfg(feature = "dim3")]
pub mod math {
    use bevy::math::{Quat, Vec3};
    /// The real type (f32 or f64).
    pub type Real = salva::math::Real;
    /// The vector type.
    pub type Vect = Vec3;
    /// The rotation type.
    pub type Rot = Quat;
}

pub mod plugin;
pub mod fluid;
pub mod rapier_integration;
pub mod utils;
