use bevy::prelude::{Component, Reflect};
use salva::{object::FluidHandle, solver::NonPressureForce};
use salva::object::interaction_groups::InteractionGroups;
use crate::math::{Real, Vect};

#[derive(Component)]
pub struct SalvaFluidHandle(pub FluidHandle);

#[derive(Component)]
pub struct FluidParticlePositions {
    pub positions: Vec<Vect>,
}

/// The rest density of a fluid (default 1000.0)
#[derive(Component)]
pub struct FluidDensity {
    pub density0: Real,
}

impl Default for FluidDensity {
    fn default() -> Self {
        Self { density0: 1000.0 }
    }
}

#[derive(Component)]
pub struct FluidNonPressureForces(pub Vec<Box<dyn NonPressureForce>>);

#[derive(Component)]
pub struct AppendNonPressureForces(pub Vec<Box<dyn NonPressureForce>>);

#[derive(Component)]
pub struct RemoveNonPressureForcesAt(pub Vec<usize>);

/// A bit mask identifying groups for fluid interactions.
#[derive(Component, Reflect, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Group(u32);

bitflags::bitflags! {
    impl Group: u32 {
        /// The group n°1.
        const GROUP_1 = 1 << 0;
        /// The group n°2.
        const GROUP_2 = 1 << 1;
        /// The group n°3.
        const GROUP_3 = 1 << 2;
        /// The group n°4.
        const GROUP_4 = 1 << 3;
        /// The group n°5.
        const GROUP_5 = 1 << 4;
        /// The group n°6.
        const GROUP_6 = 1 << 5;
        /// The group n°7.
        const GROUP_7 = 1 << 6;
        /// The group n°8.
        const GROUP_8 = 1 << 7;
        /// The group n°9.
        const GROUP_9 = 1 << 8;
        /// The group n°10.
        const GROUP_10 = 1 << 9;
        /// The group n°11.
        const GROUP_11 = 1 << 10;
        /// The group n°12.
        const GROUP_12 = 1 << 11;
        /// The group n°13.
        const GROUP_13 = 1 << 12;
        /// The group n°14.
        const GROUP_14 = 1 << 13;
        /// The group n°15.
        const GROUP_15 = 1 << 14;
        /// The group n°16.
        const GROUP_16 = 1 << 15;
        /// The group n°17.
        const GROUP_17 = 1 << 16;
        /// The group n°18.
        const GROUP_18 = 1 << 17;
        /// The group n°19.
        const GROUP_19 = 1 << 18;
        /// The group n°20.
        const GROUP_20 = 1 << 19;
        /// The group n°21.
        const GROUP_21 = 1 << 20;
        /// The group n°22.
        const GROUP_22 = 1 << 21;
        /// The group n°23.
        const GROUP_23 = 1 << 22;
        /// The group n°24.
        const GROUP_24 = 1 << 23;
        /// The group n°25.
        const GROUP_25 = 1 << 24;
        /// The group n°26.
        const GROUP_26 = 1 << 25;
        /// The group n°27.
        const GROUP_27 = 1 << 26;
        /// The group n°28.
        const GROUP_28 = 1 << 27;
        /// The group n°29.
        const GROUP_29 = 1 << 28;
        /// The group n°30.
        const GROUP_30 = 1 << 29;
        /// The group n°31.
        const GROUP_31 = 1 << 30;
        /// The group n°32.
        const GROUP_32 = 1 << 31;

        /// All of the groups.
        const ALL = u32::MAX;
        /// None of the groups.
        const NONE = 0;
    }
}

impl Default for Group {
    fn default() -> Self {
        Group::ALL
    }
}

/// Pairwise fluid particle collision filtering using bit masks.
///
/// This filtering method is based on two 32-bit values:
/// - The interaction groups memberships.
/// - The interaction groups filter.
///
/// An interaction is allowed between two filters `a` and `b` when two conditions
/// are met simultaneously:
/// - The groups membership of `a` has at least one bit set to `1` in common with the groups filter of `b`.
/// - The groups membership of `b` has at least one bit set to `1` in common with the groups filter of `a`.
///
/// In other words, interactions are allowed between two filter iff . the following condition is met:
/// ```ignore
/// (self.memberships & rhs.filter) != 0 && (rhs.memberships & self.filter) != 0
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Component, Reflect)]
pub struct FluidInteractionGroups {
    /// Groups memberships.
    pub memberships: Group,
    /// Groups filter.
    pub filters: Group,
}

impl FluidInteractionGroups {
    /// Creates a new collision-groups with the given membership masks and filter masks.
    pub const fn new(memberships: Group, filters: Group) -> Self {
        Self {
            memberships,
            filters,
        }
    }
}

impl Default for FluidInteractionGroups {
    fn default() -> Self {
        Self {
            memberships: Group::ALL,
            filters: Group::ALL,
        }
    }
}

impl From<FluidInteractionGroups> for InteractionGroups {
    fn from(groups: FluidInteractionGroups) -> Self {
        InteractionGroups {
            memberships: salva::object::interaction_groups::Group::from_bits(groups.memberships.bits())
                .unwrap(),
            filter: salva::object::interaction_groups::Group::from_bits(groups.filters.bits())
                .unwrap(),
        }
    }
}
