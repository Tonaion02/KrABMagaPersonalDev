use bevy::prelude::*;

//T: engine_configuration is substituted by simulation_descriptor
// use crate::engine::resources::engine_configuration::EngineConfiguration;

// pub fn engine_config_update(mut engine_config: ResMut<EngineConfiguration>) {
//     engine_config.current_step += 1;
// }

use crate::engine::resources::simulation_descriptor::SimulationDescriptorT;

pub fn simulation_descriptor_update_system(mut simulation_descriptor: ResMut<SimulationDescriptorT>) {
    simulation_descriptor.current_step += 1;
}