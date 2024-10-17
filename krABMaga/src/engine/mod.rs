// Bevy reexports, so that we can prevent exposing bevy directly
// TODO can we simplify those for the user without sacrificing flexibility?

// T: commented because cause errors on Component
pub use bevy::ecs as bevy_ecs;
pub use bevy::prelude::Component;
pub use bevy::prelude::Entity;
pub use bevy::prelude::Query;
pub use bevy::prelude::Res;

//T: added by me
pub use bevy::prelude::With;
pub use bevy::prelude::Without;
pub use bevy::prelude::Commands;
pub use bevy::prelude::ResMut;
pub use bevy::prelude::Resource;
pub use bevy::prelude::Update;
pub use bevy::prelude::Startup;
pub use bevy::ecs::system::ParallelCommands;
pub use bevy::prelude as bevy_prelude;

pub use bevy_utils;
pub use bevy_utils::Parallel;

/// Module to define Agent methods
pub mod agent;

/// Folder containing all the fields available on the engine
pub mod fields;
/// File to define the basic structs for locations of the agents used for Fields
pub mod location;

pub mod components;
pub mod resources;
pub mod rng;
pub mod simulation;
/// Module to define State methods
pub mod state;
pub mod systems;

pub mod thread_id;

// TODO consider removing/abstracting away

// TEMP
pub use rand::distributions::uniform::SampleRange;