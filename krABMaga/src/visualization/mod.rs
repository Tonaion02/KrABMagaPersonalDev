// pub mod agent_render;
// pub mod fields;
pub mod simulation_descriptor;
pub mod asset_handle_factory;
mod systems;

//T: verify if we need this things
// pub mod utils;
pub mod visualization;

//T: removed because we don't need anymore this things
// pub mod visualization_state;
// pub mod wrappers;

//T: added from me, export this keyword cause we need in main
//T: probably temporary until we find a better architecture solution
pub use bevy::prelude::AssetServer;
pub use bevy::prelude::Transform;
pub use bevy::sprite::SpriteBundle;
pub use bevy::prelude::Vec3;
pub use bevy::render::color::Color as Color;