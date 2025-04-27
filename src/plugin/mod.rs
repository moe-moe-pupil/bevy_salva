pub use self::plugin::{
    SalvaContextInitialization, SalvaPhysicsPlugin, SalvaSimulationSet
};
pub use crate::fluid::AppendNonPressureForces;
pub use crate::fluid::RemoveNonPressureForcesAt;
pub use configuration::*;
pub use salva_context::*;

#[allow(clippy::type_complexity)]
pub mod systems;
#[allow(clippy::module_inception)]
mod plugin;
mod salva_context;
mod configuration;