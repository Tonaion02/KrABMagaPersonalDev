use krabmaga::engine::bevy_ecs;
use krabmaga::engine::bevy_ecs::system::Resource;
use krabmaga::engine::Commands;

use krabmaga::visualization::simulation_descriptor::SimulationDescriptor;
use krabmaga::visualization::graphic_initializer::GraphicInitializer;





#[derive(Resource)]
pub struct GI;

impl GraphicInitializer for GI {

    fn on_init(
        &self,
        commands: &mut Commands,
        simulation_descriptor: &mut SimulationDescriptor
    ) {
        
    }

}