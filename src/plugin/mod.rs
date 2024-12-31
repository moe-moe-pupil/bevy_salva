pub use self::plugin::{
    SalvaContext, SalvaSimulationSet, SalvaPhysicsPlugin, AppendNonPressureForces,
    RemoveNonPressureForcesAt
};

pub mod systems;
mod plugin;