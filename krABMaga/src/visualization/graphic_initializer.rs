use bevy::ecs::system::Resource;
use bevy::prelude::Commands;

use super::simulation_descriptor::SimulationDescriptor;

pub trait GraphicInitializer: Resource {
    fn on_init(
        &self,
        commands: &mut Commands,
        simulation_descriptor: &mut SimulationDescriptor
    ) {

    }
}