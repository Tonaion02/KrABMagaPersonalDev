#![allow(warnings)]
//==============================================================================================================
//--------------------------------------------------------------------------------------------------------------
// WOLF-SHEEP-GRASS SIMULATION
//--------------------------------------------------------------------------------------------------------------
// The comment's lines that start with 'T:' are left by Tonaion02.
// The comment's lines where there is 'TODO' is a reminder for something that we must
// to do.
// The comment's lines where there is 'WARNING' represent some information that you
// have to consider to use the code in proper way.
// The comment's lines that end with '(START)' are the begin of a block of code.
// The comment's lines that end with '(END)' are the end of a block of code.
//==============================================================================================================

extern crate rayon;
use crate::rayon::iter::IntoParallelRefIterator;

use std::time::Instant;

use std::sync::Arc;
use std::sync::Mutex;


//use crate::model::state::WsgState;
// T: model's import
mod model;
use crate::model::animals::Sheep;
use crate::model::animals::Wolf;
use crate::model::animals::Location;
use crate::model::animals::LastLocation;

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
use krabmaga::engine::fields::dense_object_grid_2d_t::DenseBagGrid2D;

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

// T: debug's import
use model::debug::count_agents;
use model::debug::count_sheeps;
use model::debug::count_wolfs;
use rayon::iter::ParallelIterator;

// T: Constants(START)
pub const ENERGY_CONSUME: f32 = 1.0;

pub const FULL_GROWN: u16 = 20;

pub const GAIN_ENERGY_SHEEP: f32 = 4.0;
pub const GAIN_ENERGY_WOLF: f32 = 20.0;

pub const SHEEP_REPR: f32 = 0.2;
pub const WOLF_REPR: f32 = 0.1;

pub const MOMENTUM_PROBABILITY: f32 = 0.8;
// T: new costants(START)
pub const STEPS: u32 = 200;
pub const NUM_THREADS: usize = 4;
pub const DIM_X: f32 = 5000.;
pub const DIM_Y: f32 = DIM_X;
pub const NUM_AGENTS: f32 = 2000000.;
pub const NUM_INITIAL_SHEEPS: u32 = (NUM_AGENTS * 0.6) as u32;
pub const NUM_INITIAL_WOLFS: u32 = (NUM_AGENTS * 0.4) as u32;
// T: new costants(END)
// T: Constants(END)



// T: markers for fields
pub struct SheepField;
pub struct WolfField;

use crate::engine::fields::atomic_grid::AtomicGrid2D;
pub struct CountSheeps;


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
    // T: to add to app many systems
    // T: TODO add the system necessary to double buffer grass_field
    let app = &mut simulation.app;
    app.add_systems(Update, count_wolfs_for_location);
    app.add_systems(Update, move_agents);

    // T: TEMP
    // T: only for debug purpose
    //app.add_systems(Update, count_agents);
    
    //app.add_systems(Update, count_sheeps);
    //app.add_systems(Update, count_wolfs);

    simulation
}




// Doens't work because the number of sheep that must die must be less of this number, but not has to
// to be this number.......
// fn compute_sheeps_for_loc(query_sheeps: Query<(Entity, &Sheep, &DBRead<Location>)>, 
//                         mut query_count_grid: Query<(&AtomicGrid2D<CountSheeps>)>,
//                         mut parallel_commands: ParallelCommands) {

//     //println!("compute_sheeps_for_loc!");

//     let grid = query_count_grid.single();

//     // T: count sheeps that must die
//     // T: TODO verify if i need to check if the sheep is already died
//     query_sheeps.par_iter().for_each(|(entity, sheep_data, sheep_loc)|{
//         grid.get_ref_counter(&sheep_loc.0.0).fetch_add(1u32, std::sync::atomic::Ordering::AcqRel);

//         parallel_commands.command_scope(|mut commands| {
//             commands.entity(entity).despawn();
//         });
//     });

//     // T: TEMP debug purpose
//     let mut count = 0;
//     for element in & grid.values {
//         if element.load(std::sync::atomic::Ordering::Acquire) > 0 {
//             count += 1;
//         }
//     }

//     println!("count non zero cells: {}", count);
// }

// fn wolfs_eat(mut query_wolfs: Query<(&mut Wolf, &DBRead<Location>)>, mut query_count_grid: Query<(&AtomicGrid2D<CountSheeps>)>) {

//     //println!("wolfs_eat!");

//     let grid = query_count_grid.single();

//     // T: Update the energies of a numbers of wolf equals to the number of sheeps that died
//     query_wolfs.par_iter_mut().for_each(|(mut wolf_data, wolf_loc)|{

//         // Decrease the value in the count-sheeps-to-kill-grid
//         if grid.get_ref_counter(&wolf_loc.0.0).load(std::sync::atomic::Ordering::Acquire) > 0 {
//             grid.get_ref_counter(&wolf_loc.0.0).fetch_add(0u32, std::sync::atomic::Ordering::AcqRel);
//         }

//         // Update the energy's value of each wolf
//         wolf_data.energy += GAIN_ENERGY_WOLF;
//     });

//     // T: TEMP debug purpose
//     for element in & grid.values {
//         if element.load(std::sync::atomic::Ordering::Acquire) < 0 {
//             println!("Error, atomics doesn't work, an atomic has zero value");
//         }
//     }
// }


// 0 Wolf
// 1 Sheep
fn count_wolfs_for_location(query_wolfs: Query<(&Wolf, &DBRead<Location>)>, mut query_count_grid: Query<(&AtomicGrid2D<CountSheeps>)>) {

    let mut grid = query_count_grid.single_mut();

    query_wolfs.par_iter().for_each(|(wolf_data, wolf_loc)| {
        let binding = grid.get_atomic_counter(&wolf_loc.0.0);
        let mut counter = binding.lock().unwrap();
        counter.0 += 1;
    });

    // TEMP for debug purpose
    // let mut verify_counter = 0;
    // for element in & grid.values {
    //     verify_counter += element.lock().unwrap().0;
    // }
    // println!("total wolf in the grid: {}", verify_counter);
}

fn count_sheeps_for_location(query_sheeps: Query<(&Sheep, &DBRead<Location>)>, mut query_count_grid: Query<(&AtomicGrid2D<CountSheeps>)>) {

    let mut grid = query_count_grid.single_mut();

    query_sheeps.par_iter().for_each(|(sheep_data, sheep_loc)| {
        let binding = grid.get_atomic_counter(&sheep_loc.0.0);
        let mut counter = binding.lock().unwrap();
        counter.1 += 1;
    });

    // TEMP for debug purpose
    // let mut verify_counter = 0;
    // for element in & grid.values {
    //     verify_counter += element.lock().unwrap().1;
    // }
    // println!("total sheeps in the grid: {}", verify_counter);
}

use std::cmp::min;

fn wolfs_eat(mut query_wolfs: Query<(&mut Wolf, &DBRead<Location>)>, query_count_grid: Query<(&AtomicGrid2D<CountSheeps>)>) {

    let grid = query_count_grid.single();

    query_wolfs.par_iter_mut().for_each(|(mut wolf_data, wolf_loc)| {

        let binding = grid.get_atomic_counter(&wolf_loc.0.0);
        let mut counter = binding.lock().unwrap();
        let min = std::cmp::min(counter.0, counter.1);
        if min <= 0 {
            if counter.0 < counter.1 {
                counter.0 -= 1;
            } else {
                counter.1 -= 1;
                
            }
        }

    });
}

fn sheeps_die(query_count_grid: Query<(&AtomicGrid2D<CountSheeps>)>, mut parallel_commands: ParallelCommands) {

    let mut grid = query_count_grid.single();

    grid.values.par_iter().for_each(|(counter)|{
        parallel_commands.command_scope(|(mut commands)| {
            
        });
    });

    //let binding = grid.get_atomic_counter();
    //let mut counter = binding.lock().unwrap();
    

}

fn sheeps_eat() {

}

fn reproduce_wolfs() {



}

fn reproduce_sheeps() {

}

fn move_agents(mut query_agents: Query<(&mut DBWrite<Location>, &mut DBWrite<LastLocation>)>) {

    query_agents.par_iter_mut().for_each(|(mut loc, mut last_loc)| {

        let x = loc.0.0.x;
        let y = loc.0.0.y;
        let mut rng = rand::thread_rng();

        let mut moved = false;
        if last_loc.0.0.is_some() && rng.gen_bool(MOMENTUM_PROBABILITY as f64) {
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












fn init_world(mut commands: Commands) {
    println!("init_world!");

    let mut rng = rand::thread_rng();

    // T: generate sheeps (START)
    println!("generate sheeps");

    for sheep_id in 0..NUM_INITIAL_SHEEPS {

        let loc = Int2D { x: rng.gen_range(0..DIM_X as i32), y: rng.gen_range(0..DIM_Y as i32) };
        let initial_energy = rng.gen_range(0.1 ..(2. * GAIN_ENERGY_SHEEP as f32));
        //println!("{}", initial_energy);

        let entity_commands = commands.spawn((

            Sheep {
                id: sheep_id + NUM_INITIAL_WOLFS,
                energy: initial_energy as f32,
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
        let initial_energy = rng.gen_range(0.1 ..(2. * GAIN_ENERGY_SHEEP as f32));

        let entity_command = commands.spawn(
            
    (Wolf {
                id: wolf_id,
                energy: initial_energy as f32,
            }, 

            DoubleBuffered::new(Location(loc)),
            DoubleBuffered::new(LastLocation(None)),

            Agent,
        ));
    }

    let mut counters_field = AtomicGrid2D::<CountSheeps>::new((0u32, 0u32), DIM_X as i32, DIM_Y as i32);

    // let mut index = 0;
    // for counter in &mut counters_field.values {
    //     print!("({},{}): {}", index%counters_field.height, index/counters_field.height, counter.load(std::sync::atomic::Ordering::Acquire));

    //     index += 1;
    // }


    commands.spawn((counters_field));
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