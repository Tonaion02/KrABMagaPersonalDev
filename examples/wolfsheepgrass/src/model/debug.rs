use krabmaga::engine::components::double_buffer::DoubleBuffered;
use krabmaga::engine::components::double_buffer::DBRead;
use krabmaga::engine::components::double_buffer::DBWrite;

use krabmaga::engine::agent::Agent;

use krabmaga::engine::resources::simulation_descriptor::SimulationDescriptorT;

use crate::model::animals::Sheep;
use crate::model::animals::Wolf;
use crate::model::animals::Location;
use crate::model::animals::LastLocation;

// T: bevy's import
// T: TODO find a way to remove the necessity to use this tools
use krabmaga::engine::Commands;
use krabmaga::engine::Query;
use krabmaga::engine::Update;
use krabmaga::engine::Entity;
use krabmaga::engine::bevy_ecs as bevy_ecs;
use krabmaga::engine::Component;
use krabmaga::engine::bevy_ecs::prelude::EntityWorldMut;
use krabmaga::engine::ParallelCommands;
use krabmaga::engine::Without;
use krabmaga::engine::bevy_prelude::*;





// T: TEMP
// T: TODO trying to use the fucking AgentFactory that probably is the best
// T: idea.
pub fn insert_double_buffered<T: Component + Copy>(mut entity: EntityWorldMut, value: T) {
    entity.insert(DoubleBuffered::new(value));
}

// T: TEMP
// T: For debug purpose
pub fn count_agents(query_agents: Query<(&Agent)>) {

    let mut count = 0u32;

    query_agents.for_each(|(agent)| {
        count = count + 1;
    });

    println!("{}", count);
}

// T: TEMP
// T: For debug purpose
pub fn count_wolfs(query_wolfs: Query<(&Wolf)>) {

    let mut count = 0u32;

    query_wolfs.for_each(|(sheep)|{
        count = count + 1;
    });

    println!("Wolfs: {}", count);
}

// T: TEMP
// T: For debug purpose
pub fn count_sheeps(query_sheeps: Query<(&Sheep)>) {

    let mut count = 0u32;

    query_sheeps.for_each(|(wolf)| {
        count = count + 1;
    });

    println!("Sheeps: {}", count);
}

// T: TEMP
// T: For debug purpose
pub fn print_step(simulation_descriptor: Res<SimulationDescriptorT>) {
    println!("---------------------STEP---------------->: {}", simulation_descriptor.current_step);
}