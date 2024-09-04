//==============================================================================================================
//--------------------------------------------------------------------------------------------------------------
// WOLF-SHEEP-GRASS SIMULATION
//--------------------------------------------------------------------------------------------------------------
// The comment's lines that start with 'T:' are left by Tonaion02.
// The comment's lines where there is 'TODO' is a reminder for something that we must
// to do.
// The comment's lines where there is 'NOTE' represent some information that you
// have to know.
// The comment's lines that end with '(START)' are the begin of a block of code.
// The comment's lines that end with '(END)' are the end of a block of code.
//==============================================================================================================
use crate::model::state::WsgState;
mod model;

use engine::location::Real2D;
use krabmaga::engine::simulation::Simulation;

// T: Constants(START)
pub const ENERGY_CONSUME: f64 = 1.0;

pub const FULL_GROWN: u16 = 20;

pub const GAIN_ENERGY_SHEEP: f64 = 4.0;
pub const GAIN_ENERGY_WOLF: f64 = 20.0;

pub const SHEEP_REPR: f64 = 0.2;
pub const WOLF_REPR: f64 = 0.1;

pub const MOMENTUM_PROBABILITY: f64 = 0.8;

// T: new costants(START)
pub const STEPS: i32 = 200;
pub const NUM_THREADS: u32 = 4;
pub const DIM_X: u32 = 50;
pub const DIM_Y: u32 = DIM_X;
pub const INITIAL_SHEEPS: u32 = (200. * 0.6) as u32;
pub const INITIAL_WOLFS: u32 = (200. * 0.4) as u32;
// T: new costants(END)
// T: Constants(END)





// 'No-visualization' specific imports
#[cfg(not(any(feature = "visualization", feature = "visualization_wasm")))]
use krabmaga::*;

#[cfg(not(any(feature = "visualization", feature = "visualization_wasm")))]
fn main() {
    // T: why steps are not a constant?
    // T: probably they cannot be only a constant cause the fact
    // T: that we need to access to steps from other sources
    let step = 200;

    // T: why dim are not a constant?
    // T: same that above with steps
    let dim: (i32, i32) = (50, 50);
    // T: same that above
    let initial_animals: (u32, u32) = ((200. * 0.6) as u32, (200. * 0.4) as u32);

    // T: commented because out-dated
    // let state = WsgState::new(dim, initial_animals);
    // let _ = simulate!(state, step, 10);
    let simulation = build_simulation();
    simulation.run();
}

fn build_simulation() -> Simulation {
    
    let simulation = Simulation::build();
    simulation.with_steps(STEPS);
    simulation.with_num_threads(NUM_THREADS);
    simulation.with_simulation_dim(Real2D {x: DIM_X, y: DIM_Y});

    //Add the components that must be double buffered
    

    simulation
}

fn sheep_step() {

}

fn wolf_step() {

}

fn grass_step() {

}

#[cfg(any(feature = "visualization", feature = "visualization_wasm"))]
mod visualization;

#[cfg(any(feature = "visualization", feature = "visualization_wasm"))]
use {
    crate::visualization::vis_state::VisState, krabmaga::bevy::prelude::Color,
    krabmaga::engine::fields::dense_number_grid_2d::DenseNumberGrid2D,
    krabmaga::visualization::fields::number_grid_2d::BatchRender,
    krabmaga::visualization::visualization::Visualization,
};

// Main used when a visualization feature is applied
#[cfg(any(feature = "visualization", feature = "visualization_wasm"))]
fn main() {
    let dim: (i32, i32) = (25, 25);

    let initial_animals: (u32, u32) = ((60. * 0.6) as u32, (60. * 0.4) as u32);

    let state = WsgState::new(dim, initial_animals);
    let mut app = Visualization::default()
        .with_background_color(Color::rgb(255., 255., 255.))
        .with_simulation_dimensions(dim.0 as f32, dim.1 as f32)
        .with_window_dimensions(1000., 700.)
        .setup::<VisState, WsgState>(VisState, state);
    app.add_system(DenseNumberGrid2D::batch_render);
    app.run()
}
