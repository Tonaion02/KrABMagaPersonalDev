use std::default::Default;

use bevy::prelude::Resource;

use crate::engine::location::Real2D;





// T: A struct that contains the data necessary to describe a simulation
#[derive(Resource)]
pub struct SimulationDescriptorT {

    pub current_step: u64,
    pub simulation_dim: Real2D,
    pub paused: bool,
    pub rand_seed: u64,

    pub steps: Option<u32>,
    pub num_threads: usize,

    // T: added to support fixed random(to put in RNG)
    pub seed: u64,

    pub title: String,
}

impl Default for SimulationDescriptorT {

    fn default() -> SimulationDescriptorT {
        SimulationDescriptorT {
            current_step: 0,
            simulation_dim: Real2D { x: 0., y: 0. },
            paused: true,
            rand_seed: 0,

            steps: None,
            num_threads: 1,

            seed: 0,
            
            title: String::from("default-simulation-title"),
        }
    }

}