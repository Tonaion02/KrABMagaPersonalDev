#![allow(warnings)]
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

use std::time::Instant;

//use crate::model::state::WsgState;
mod model;
use crate::model::animals::Sheep;
use crate::model::animals::Wolf;
use crate::model::animals::Location;
use crate::model::animals::LastLocation;
use crate::model::animals::Energy;

use engine::agent;
use engine::components::double_buffer::DBClonableRead;
use engine::components::double_buffer::DBClonableWrite;
use engine::components::double_buffer::DoubleBufferedDataStructure;
use krabmaga::engine::components::double_buffer::DoubleBuffered;
use krabmaga::engine::components::double_buffer::DBRead;
use krabmaga::engine::components::double_buffer::DBWrite;
use krabmaga::rand::Rng;

use engine::location::Real2D;
use engine::location::Int2D;
use krabmaga::engine::simulation::Simulation;
use krabmaga::engine::agent::Agent;
use krabmaga::engine::fields::dense_number_grid_2d_t::DenseSingleValueGrid2D;

// T: bevy's import
// T: TODO find a way to remove the necessity to use this tools
use krabmaga::engine::Commands;
use krabmaga::engine::Query;
use krabmaga::engine::Update;

use krabmaga::engine::bevy_ecs as bevy_ecs;
use krabmaga::engine::Component;
use krabmaga::engine::bevy_ecs::prelude::EntityWorldMut;


// T: Constants(START)
pub const ENERGY_CONSUME: f64 = 1.0;

pub const FULL_GROWN: u16 = 20;

pub const GAIN_ENERGY_SHEEP: f64 = 4.0;
pub const GAIN_ENERGY_WOLF: f64 = 20.0;

pub const SHEEP_REPR: f64 = 0.2;
pub const WOLF_REPR: f64 = 0.1;

pub const MOMENTUM_PROBABILITY: f64 = 0.8;
// T: new costants(START)
pub const STEPS: u32 = 1;
pub const NUM_THREADS: usize = 4;
pub const DIM_X: f32 = 50.;
pub const DIM_Y: f32 = DIM_X;
pub const NUM_INITIAL_SHEEPS: u32 = (200. * 0.6) as u32;
pub const NUM_INITIAL_WOLFS: u32 = (200. * 0.4) as u32;
// T: new costants(END)
// T: Constants(END)





// 'No-visualization' specific imports
#[cfg(not(any(feature = "visualization", feature = "visualization_wasm")))]
use krabmaga::*;

#[cfg(not(any(feature = "visualization", feature = "visualization_wasm")))]
fn main() {

    let now = Instant::now();

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

    let elapsed_time = now.elapsed();
    println!("Elapsed: {:.2?}, steps per second: {}", elapsed_time, STEPS as f64 / elapsed_time.as_secs_f64());
    save_elapsed_time(elapsed_time);
}

fn build_simulation() -> Simulation {
    
    let mut simulation = Simulation::build();
    simulation = simulation
    .with_steps(STEPS)
    .with_num_threads(NUM_THREADS)
    .with_simulation_dim(Real2D {x: DIM_X, y: DIM_Y});

    //Add the components that must be double buffered
    simulation = simulation
    .register_double_buffer::<LastLocation>()
    .register_double_buffer::<Location>();
    
    simulation = simulation.register_init_world(init_world);

    // T: TEMP
    // T: TODO create some abstractions of Simulation that permits
    // T: to add to app many other systems
    // T: TODO add the system necessary to double buffer grass_field
    let app = &mut simulation.app;
    app.add_systems(Update, move_agents);
    app.add_systems(Update, sheeps_eat);
    app.add_systems(Update, wolfs_eat);
    app.add_systems(Update, grass_grow);

    simulation
}


// T: TEMP
// T: TODO trying to use the fucking AgentFactory that probably is the best
// T: idea.
pub fn insert_double_buffered<T: Component + Copy>(mut entity: EntityWorldMut, value: T) {
    entity.insert(DoubleBuffered::new(value));
}

fn init_world(mut commands: Commands) {

    println!("init_world!");

    // T: generate grass (START)
    println!("generate grass");

    let mut grass_field = DenseSingleValueGrid2D::<u16>::new(DIM_X as i32, DIM_Y as i32);

    (0..DIM_X as i32).into_iter().for_each(|x| {
        (0..DIM_Y as i32).into_iter().for_each(|y| {
            let mut rng = rand::thread_rng();
            let fully_growth = rng.gen_bool(0.5);
            if fully_growth {

                // T: TODO add the missing code with DenseGrid for Grass
                grass_field.set_value_location(FULL_GROWN, &Int2D { x, y });
            } else {
                let grass_init_value = rng.gen_range(0..FULL_GROWN + 1);

                grass_field.set_value_location(grass_init_value, &Int2D { x, y })
            }
        })
    });

    commands.spawn((DoubleBufferedDataStructure::new(grass_field)));
    // T: generate grass (END)

    // T: generate sheeps (START)
    println!("generate sheeps");
    let mut rng = rand::thread_rng();
    for sheep_id in 0..NUM_INITIAL_SHEEPS {

        let loc = Int2D { x: rng.gen_range(0..DIM_X as i32), y: rng.gen_range(0..DIM_Y as i32) };
        let initial_energy = rng.gen_range(0..(2 * GAIN_ENERGY_SHEEP as usize));

        commands.spawn((

            Sheep {
                id: sheep_id + NUM_INITIAL_WOLFS,
            }, 
            
            Energy {
                energy: initial_energy as f64,
            },

            DoubleBuffered::new(Location(loc)),
            DoubleBuffered::new(LastLocation(None)),

            Agent,

        ));
    }
    // T: generate sheeps (END)

    // T: generate wolfs (START)
    println!("genereate wolfs");
    for wolf_id in 0..NUM_INITIAL_WOLFS {

        let loc = Int2D { x: rng.gen_range(0..DIM_X as i32), y: rng.gen_range(0..DIM_Y as i32) };
        let initial_energy = rng.gen_range(0..(2 * GAIN_ENERGY_SHEEP as usize));

        commands.spawn(
            
    (Wolf {
                id: wolf_id,
            }, 

            Energy {
                energy: initial_energy as f64,
            },

            DoubleBuffered::new(Location(loc)),
            DoubleBuffered::new(LastLocation(None)),

            Agent,
        ));
    }
    // T: generate wolfs (END)
}

// T: TODO register modifies in the fields(or add function to recreate fields)
fn move_agents(mut query_agents: Query<(&mut DBWrite<Location>, &mut DBWrite<LastLocation>)>) {

    query_agents.par_iter_mut().for_each(|(mut loc, mut last_loc)| {
        
        let x = loc.0.0.x;
        let y = loc.0.0.y;
        let mut rng = rand::thread_rng();

        let mut moved = false;
        if last_loc.0.0.is_some() && rng.gen_bool(MOMENTUM_PROBABILITY) {
            if let Some(pos) = last_loc.0.0 {
                let xm = x + (x - pos.x);
                let ym = y + (y - pos.y);
                let new_loc = Int2D { x: xm, y: ym };
                // TRY TO MOVE WITH MOMENTUM_PROBABILITY
                if xm >= 0 && xm < DIM_X as i32 && ym >= 0 && ym < DIM_Y as i32 {
                    loc.0 = Location(new_loc);
                    last_loc.0 = LastLocation(Some(Int2D { x, y }));
                    moved = true;
                }
            }
        }



        if !moved {
            let xmin = if x > 0 { -1 } else { 0 };
            let xmax = i32::from(x < DIM_X as i32 - 1);
            let ymin = if y > 0 { -1 } else { 0 };
            let ymax = i32::from(y < DIM_Y as i32 - 1);

            // let nx = if rng.gen_bool(0.5) { xmin } else { xmax };
            // let ny = if rng.gen_bool(0.5) { ymin } else { ymax };
            let nx = rng.gen_range(xmin..=xmax);
            let ny = rng.gen_range(ymin..=ymax);

            // T: OLD
            // self.loc = Int2D {
            //     x: x + nx,
            //     y: y + ny,
            // };
            // self.last = Some(Int2D { x, y });

            loc.0 = Location(Int2D { x: x + nx, y: y + ny, });
            last_loc.0 = LastLocation(Some(Int2D { x, y }));
        }
    });

}

// T: TODO check if it is necessary to make double buffered the energy of a sheep
fn sheeps_eat(mut query_sheeps: Query<(&Sheep, &mut Energy,&DBRead<Location>)>, 
              mut query_grass_field: Query<(&DBClonableRead<DenseSingleValueGrid2D<u16>>, &mut DBClonableWrite<DenseSingleValueGrid2D<u16>>)>) {

    let mut grass_fields = query_grass_field.get_single_mut().expect("msg");
    let read_grass_field = grass_fields.0;
    let mut write_grass_field = grass_fields.1;

    query_sheeps.iter_mut().for_each(|(mut sheep, loc)| {
        
        // T: TODO check if it is necessary to check that value is not written in these iteration or we
        // T: can work on the old iteration values
        // T: I don't know exactly what is the limit on working with the old iteration
        // T: to not create inconsistent situations
        // T: these is particular difficult to parallelize, for what we must use DoubleBuffering?
        if let Some(grass_value) = read_grass_field.0.get_value(&loc.0.0) {
            // T: Why >= and not = ???
            if grass_value >= FULL_GROWN {
                write_grass_field.0.set_value_location(0, &loc.0.0);
                 += GAIN_ENERGY_SHEEP;
            }
        }
    });
}

fn wolfs_eat(mut query_wolfs: Query<(&mut Wolf)>, mut commands: Commands) {



}

fn reproduce_sheeps(mut query_sheeps: Query<()>, mut commands: Commands) {

}

fn reproduce_wolves() {

}

// T: TODO check if we need double buffering for grass field
fn grass_grow(mut query_grass_field: Query<(&mut DBClonableWrite<DenseSingleValueGrid2D<u16>>)>) {

    let mut grass_field = &mut query_grass_field.single_mut().0;

    let closure = |grass_value: &u16| { 
        let growth = *grass_value;
        if growth < FULL_GROWN {
            growth + 1
        }
        else {
            growth
        }
    };

    grass_field.apply_to_all_values(closure);
}

// T: TODO add a function to sync the grass field
fn sync_grass_field() {

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

// T: TODO check what macro make this work before ECS experiment
fn save_elapsed_time(elapsed_time: core::time::Duration) {
    
    use std::path::Path;
    use std::fs::File;
    use std::io::prelude::*;
    
    //Write on file the elapsed time
    let path = Path::new("C:/source/Python/automaticKrABMagaTesting/garbage/elapsed_time.txt");
    let display = path.display();

    // Open a file in write-only mode
    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}", display, why),
        Ok(file) => file,
    };

    let mut elapsed_time_s: String = String::from("elapsed_time=");
    elapsed_time_s.push_str(&elapsed_time.as_nanos().to_string());

    match file.write_all(elapsed_time_s.as_bytes()) {
        Err(why) => panic!("couldn't write to {}: {}", display, why),
        Ok(_) => println!("successfully wrote to {}", display),
    }
    //Write on file the elapsed time
}