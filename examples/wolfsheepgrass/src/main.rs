#![allow(warnings)]
//==============================================================================================================
//--------------------------------------------------------------------------------------------------------------
// WOLF-SHEEP-GRASS SIMULATION
//--------------------------------------------------------------------------------------------------------------
// STEPS NUM_AGENTS NUM_THREADS PERC_WOLF PERC_SHEEPS
// 
//
//--------------------------------------------------------------------------------------------------------------
// The comment's lines that start with 'T:' are left by Tonaion02.
// The comment's lines where there is 'TODO' is a reminder for something that we must
// to do.
// The comment's lines where there is 'WARNING' represent some information that you
// have to consider to use the code in proper way.
// The comment's lines that end with '(START)' are the begin of a block of code.
// The comment's lines that end with '(END)' are the end of a block of code.
//--------------------------------------------------------------------------------------------------------------
//==============================================================================================================

// T: importing rayon (START)
extern crate rayon;
use crate::rayon::iter::IntoParallelRefIterator;
use crate::rayon::iter::IntoParallelRefMutIterator;
use crate::rayon::iter::ParallelIterator;
use crate::rayon::iter::IndexedParallelIterator;
// T: importing rayon (END)

use std::env::consts::EXE_SUFFIX;
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
use crate::model::animals::Agent;

use engine::agent;
use engine::bevy_ecs::query;
use engine::components::double_buffer::DBClonableRead;
use engine::components::double_buffer::DBClonableWrite;
use engine::components::double_buffer::DoubleBufferedDataStructure;
use engine::resources::simulation_descriptor;
use engine::simulation;
use krabmaga::engine::components::double_buffer::DoubleBuffered;
use krabmaga::engine::components::double_buffer::DBRead;
use krabmaga::engine::components::double_buffer::DBWrite;

#[cfg(not(any(feature="fixed_random")))]
use krabmaga::rand::Rng;

use engine::location::Real2D;
use engine::location::Int2D;
use krabmaga::engine::simulation::Simulation;
use krabmaga::engine::fields::dense_number_grid_2d_t::DenseSingleValueGrid2D;
use krabmaga::engine::fields::dense_object_grid_2d_t::DenseBagGrid2D;

use krabmaga::engine::resources::simulation_descriptor::SimulationDescriptorT;

use krabmaga::engine::simulation::SimulationSet::Step;
use krabmaga::engine::simulation::SimulationSet::AfterStep;
use krabmaga::engine::simulation::SimulationSet::BeforeStep;

// T: TODO verify if it is useless
use krabmaga::engine::rng::RNG;
use krabmaga::engine::SampleRange;

// T: bevy's import (START)
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
// T: bevy's import (START)

// T: debug's import (START)
use model::debug::count_agents;
use model::debug::count_sheeps;
use model::debug::count_wolfs;
use model::debug::population_debug_info;
use model::debug::print_step;
// T: debug's import (END)

// T: Constants(START)
pub const ENERGY_CONSUME: f64 = 1.0;

pub const FULL_GROWN: u16 = 20;

pub const GAIN_ENERGY_SHEEP: f64 = 4.0;
pub const GAIN_ENERGY_WOLF: f64 = 20.0;

pub const SHEEP_REPR: f64 = 0.2;
pub const WOLF_REPR: f64 = 0.1;

pub const MOMENTUM_PROBABILITY: f32 = 0.8;
// T: new costants(START)
pub const STEPS: u32 = 100;
pub const NUM_THREADS: usize = 4;
pub const DIM_X: f64 = 50.;
pub const DIM_Y: f64 = DIM_X;
pub const NUM_AGENTS: f64 = 20.;
pub const PERC_SHEEPS: f64 = 0.6;
pub const PERC_WOLFS: f64 = 0.4;
pub const NUM_INITIAL_SHEEPS: u64 = (NUM_AGENTS * PERC_SHEEPS) as u64;
pub const NUM_INITIAL_WOLFS: u64 = (NUM_AGENTS * PERC_WOLFS) as u64;
pub const SEED: u64 = 21382193872;
// T: new costants(END)
// T: Constants(END)





// T: markers for fields
pub struct SheepField;
pub struct WolfField;

use crate::engine::fields::atomic_grid::AtomicGrid2D;
pub struct CountWolfs;

#[derive(Component)]
#[component(storage="SparseSet")]
pub struct Death;





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
    .with_simulation_dim(Real2D {x: DIM_X as f32, y: DIM_Y as f32}) 
    .with_seed(SEED);

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
    
    app.add_systems(Update, step.in_set(Step));

    // Must run after the despawning of entities
    app.add_systems(Update, count_wolfs_for_location.in_set(BeforeStep));
    app.add_systems(Update, update_sheeps_field.in_set(BeforeStep));
    app.add_systems(Update, grass_grow.in_set(BeforeStep));
    
    // app.add_systems(Update, count_agents.in_set(BeforeStep));
    // app.add_systems(Update, count_sheeps.in_set(BeforeStep));
    // app.add_systems(Update, count_wolfs.in_set(BeforeStep));
    app.add_systems(Update, population_debug_info.in_set(BeforeStep).before(count_wolfs_for_location).before(update_sheeps_field).before(grass_grow));
    app.add_systems(Update, print_step.in_set(BeforeStep).before(population_debug_info).before(count_wolfs_for_location).before(update_sheeps_field).before(grass_grow));

    simulation
}





// Unique step function
fn step ( 
    mut query_grass_field: Query<&mut DenseSingleValueGrid2D<u16>>,
    mut query_sheeps: Query<(Entity, &mut Sheep, &DBRead<Location>)>,

    mut query_agents: Query<(&Agent, &mut DBWrite<Location>, &mut DBWrite<LastLocation>)>,
    
    mut query_count_grid: Query<(&AtomicGrid2D<CountWolfs>)>,
    query_sheeps_field: Query<(&DenseBagGrid2D<Entity, SheepField>)>,

    mut query_wolfs: Query<(Entity, &mut Wolf, &DBRead<Location>)>,

    mut parallel_commands: ParallelCommands,

    simulation_descriptor: Res<SimulationDescriptorT>,
)
{
    let mut grass_field = query_grass_field.single_mut();



    // T: move agents (START)
    let span = info_span!("move agents");
    let span = span.enter();

    #[cfg(any(feature="debug_support"))]
    let mut count_moved_agents = 0u64;

    query_agents.par_iter_mut().for_each(|(agent, mut loc, mut last_loc)|{

        // #[cfg(any(feature="debug_support"))]
        // { count_moved_agents = count_moved_agents + 1; }

        let x = loc.0.0.x;
        let y = loc.0.0.y;

        let mut moved = false;

        #[cfg(any(feature="fixed_random"))]
        let mut rng = RNG::new(simulation_descriptor.rand_seed, simulation_descriptor.current_step);
        #[cfg(not(any(feature="fixed_random")))]
        let mut rng = rand::thread_rng();
        
        let gen_bool = rng.gen_bool(MOMENTUM_PROBABILITY as f64);

        if last_loc.0.0.is_some() && gen_bool {
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

    std::mem::drop(span);

    #[cfg(any(feature="debug_support"))]
    println!("moved_agents: {}", count_moved_agents);
    // T: move agents (END)



    // T: values of the grass before sheeps eat from that
    #[cfg(any(feature = "debug_support"))]
    println!("grass full grown before sheeps: {}", count_grass(&grass_field));
    
    #[cfg(any(feature = "debug_support"))]
    let mut sheeps_that_eaten = 0u32;

    // T: Sheeps eat (START)
    let span = info_span!("sheeps eats");
    let span = span.enter();

    query_sheeps.iter_mut().for_each(|(entity, mut sheep_data, sheep_loc)|{
        if grass_field.get_value(&sheep_loc.0.0).expect("empty cell(not possible!)") >= FULL_GROWN {
            grass_field.set_value_location(0, &sheep_loc.0.0);
            
            sheep_data.energy += GAIN_ENERGY_SHEEP;

            #[cfg(any(feature = "debug_support"))]
            { sheeps_that_eaten += 1; }
        }
    });

    std::mem::drop(span);
    // T: Sheeps eat (END)

    // T: Sheeps that has eaten 
    #[cfg(any(feature = "debug_support"))]
    println!("sheeps that has eaten: {}", sheeps_that_eaten);

    // T: Values of the grass after sheeps eat from that
    #[cfg(any(feature = "debug_support"))]
    println!("grass full grown after sheeps: {}", count_grass(&grass_field));



    // T: Sheeps reproduce (START)
    #[cfg(any(feature = "debug_support"))]
    let mut counter_for_coin_flip = Arc::new(Mutex::new(0u64));
    #[cfg(any(feature = "debug_support"))]
    let mut counter_dead_for_out_energy_sheeps = Arc::new(Mutex::new(0u64));;

    let span = info_span!("sheeps reproduce");
    let span = span.enter();

    query_sheeps.par_iter_mut().for_each(|(entity, mut sheep_data, loc)| {

        sheep_data.energy -= ENERGY_CONSUME;

        parallel_commands.command_scope(|mut commands| {

            #[cfg(not(any(feature="fixed_random")))]
            let mut rng = rand::thread_rng();
            #[cfg(any(feature="fixed_random"))]
            let mut rng = RNG::new(simulation_descriptor.rand_seed, simulation_descriptor.current_step);

            let coin_flip = rng.gen_bool(SHEEP_REPR as f64);

            #[cfg(any(feature = "debug_support"))]
            if coin_flip && sheep_data.energy > 0. {
                // counter_for_coin_flip += 1;

                let mut binding = counter_for_coin_flip.lock().unwrap();
                *binding += 1;
            }

            if sheep_data.energy > 0. && coin_flip {

                sheep_data.energy /= 2.0;

                commands.spawn((
                    Sheep {
                        id: 0,
                        energy: sheep_data.energy,
                    },

                    DoubleBuffered::new(Location(loc.0.0)),
                    DoubleBuffered::new(LastLocation(None)),

                    Agent {id: 0},
                ));

            }
            if sheep_data.energy <= 0. {
                commands.entity(entity).despawn();

                #[cfg(any(feature = "debug_support"))]
                {   
                    let mut binding = counter_dead_for_out_energy_sheeps.lock().unwrap();
                    *binding += 1;
                }
            }

        });

    });

    std::mem::drop(span);

    #[cfg(any(feature = "debug_support"))]
    let binding = counter_for_coin_flip.lock().unwrap();
    #[cfg(any(feature = "debug_support"))]
    println!("borned sheeps: {}", *binding);
    #[cfg(any(feature = "debug_support"))]
    let binding = counter_dead_for_out_energy_sheeps.lock().unwrap();
    #[cfg(any(feature = "debug_support"))]
    println!("dead sheeps for out of energy {}", *binding);
    // T: Sheeps reproduce (END)



    // T: Wolves eat (START)

    // T: TEST if at least a counter is modified

    let span = info_span!("wolfs eat");
    let span = span.enter();

    // T: sheeps die from wolfs (START)
    let span_internal = info_span!("sheeps die from wolfs");
    let span_internal = span_internal.enter();

    let mut grid = query_count_grid.single_mut();
    let sheeps_field = query_sheeps_field.single();
    let non_mut_query_sheeps = query_sheeps;

    // T: TEMP debug purpose
    #[cfg(any(feature = "debug_support"))]
    let mut counter_non_zero_counters = 0u32;
    #[cfg(any(feature = "debug_support"))]
    for counter in & grid.values {
        if *counter.lock().unwrap() > 0 {
            counter_non_zero_counters += 1;
        }
    }
    #[cfg(any(feature = "debug_support"))]
    println!("counter_non_zero_counters: {}", counter_non_zero_counters);
    // T: TEMP debug purpose

    let grid_par_iterator = grid.values.par_iter();
    sheeps_field.bags.par_iter().zip(grid_par_iterator).for_each(|(bag, binding)|{
        let mut counter = binding.lock().unwrap();
        let min = std::cmp::min(*counter as usize, bag.len());
        *counter = min as u32;

        let mut effectively_alive_sheeps = 0;
        parallel_commands.command_scope(|mut commands: Commands| {
            // for i in 0..min {
            //     if non_mut_query_sheeps.get(bag[i]).expect("not found entity during sheeps die").1.energy > 0. {
            //         commands.entity(bag[i]).despawn();

            //         effectively_alive_sheeps += 1; 
            //     }
            // }

            // for i in 0..bag.len() {
            //     if non_mut_query_sheeps.get(bag[i]).expect("error not found entity").1.energy > 0. {
            //         commands.entity(bag[i]).despawn();

            //         effectively_alive_sheeps += 1;
            //     }

            //     if effectively_alive_sheeps == min as u32 {
            //         break;
            //     }
            // }

            for element in bag {
                if non_mut_query_sheeps.get(*element).expect("not fodun entity during sheeps die").1.energy > 0. {
                    if effectively_alive_sheeps == min as u32 {
                        break;
                    }

                    commands.entity(*element).despawn();

                    effectively_alive_sheeps += 1;
                }
            }
        });

        *counter = effectively_alive_sheeps;
    });

    std::mem::drop(span_internal);
    // T: sheeps die from wolfs (END)



    let span_internal = info_span!("wolfs effectively eating");
    let span_internal = span_internal.enter();

    query_wolfs.par_iter_mut().for_each(|(entity, mut wolf_data, wolf_loc)| {
        let binding = grid.get_atomic_counter(&wolf_loc.0.0);
        let mut counter = binding.lock().unwrap();
        let sheeps_for_bag = sheeps_field.get_ref_bag(&wolf_loc.0.0).len();
        if *counter > 0 {
            wolf_data.energy += GAIN_ENERGY_WOLF;
            *counter -= 1;
        }
    });

    std::mem::drop(span_internal);

    std::mem::drop(span);
    // T: Wolves eat (END)





    // // T: Wolves eat (START)
    
    // let span = info_span!("wolfs eat");
    // let span = span.enter();


    
    // std::mem::drop(span);


    // // T: Wolves eat (END)





    // T: Reproduce wolves (START)

    #[cfg(any(feature = "debug_support"))]
    let mut count_dead_wolfs = Arc::new(Mutex::new(0u64));

    #[cfg(any(feature = "debug_support"))]
    let mut count_borned_wolfs = Arc::new(Mutex::new(0u64));

    let span = info_span!("reproducing wolfs");
    let span = span.enter();

    query_wolfs.par_iter_mut().for_each(
        |(entity, mut wolf_data, loc)| {
                  wolf_data.energy -= ENERGY_CONSUME;

                  parallel_commands.command_scope(|mut commands| {
      
                
                #[cfg(not(any(feature="fixed_random")))]
                let mut rng_div = rand::thread_rng(); 
                #[cfg(any(feature="fixed_random"))]
                let mut rng_div = RNG::new(simulation_descriptor.rand_seed, simulation_descriptor.current_step);
                
                

                let gen_bool = rng_div.gen_bool(WOLF_REPR as f64);

                    #[cfg(any(feature = "debug_support"))]
                    if wolf_data.energy > 0. && gen_bool {
                        let mut binding = count_borned_wolfs.lock().unwrap();
                        *binding += 1;
                    }

                      if wolf_data.energy > 0. && gen_bool {

                          wolf_data.energy /= 2.0;
                          commands.spawn((
                          Wolf {
                              id: 0,
                              energy: wolf_data.energy,
                          }, 
              
                          DoubleBuffered::new(Location(loc.0.0)),
                          DoubleBuffered::new(LastLocation(None)),
              
                          Agent {id: 0,},)
                          );
                      }
                      if wolf_data.energy <= 0. {
                          commands.entity(entity).despawn();

                          #[cfg(any(feature = "debug_support"))]
                          {
                            let mut binding = count_dead_wolfs.lock().unwrap(); 
                            *binding += 1; 
                          }
                      }
                      
                  });
              }
          );

    #[cfg(any(feature = "debug_support"))]
    let binding = count_borned_wolfs.lock().unwrap();
    #[cfg(any(feature = "debug_support"))]
    println!("borned wolfs: {}", *binding);      
    #[cfg(any(feature = "debug_support"))]
    let binding = count_dead_wolfs.lock().unwrap();
    #[cfg(any(feature = "debug_support"))]
    println!("dead wolfs: {}", *binding);
        

    std::mem::drop(span);
    // T: Reproduce wolves (END)
}



// Run before step
// Run at the start, like an update of the fields.......(hoping that can work)
fn count_wolfs_for_location(query_wolfs: Query<(&Wolf, &DBWrite<Location>)>, mut query_count_grid: Query<(&AtomicGrid2D<CountWolfs>)>) {

    let mut grid = query_count_grid.single_mut();

    query_wolfs.par_iter().for_each(|(wolf_data, wolf_loc)| {
        let binding = grid.get_atomic_counter(&wolf_loc.0.0);
        let mut counter = binding.lock().unwrap();
        *counter += 1;
    });

    // // TEMP for debug purpose
    // #[cfg(any(feature = "debug_support"))]
    // let mut verify_counter = 0;
    // #[cfg(any(feature = "debug_support"))]
    // for element in & grid.values {
    //     verify_counter += *element.lock().unwrap();
    // }
    // #[cfg(any(feature = "debug_support"))]
    // println!("total wolf in the grid: {}", verify_counter);
    // // TEMP for debug purpose

    // // TEMP for debug purpose
    // #[cfg(any(feature="debug_support"))]
    // let mut counter_of_location = 0;
    // #[cfg(any(feature="debug_support"))]
    // let location = Int2D {x: 49, y: 49};
    // #[cfg(any(feature="debug_support"))]
    // query_wolfs.iter().for_each(|(wolf_data, wolf_loc)| {
    //     if wolf_loc.0.0 == location {
    //         counter_of_location += 1;
    //     }
    // });
    
    // #[cfg(any(feature="debug_support"))]
    // println!("counter_of_location: {}", counter_of_location);    

    // #[cfg(any(feature="debug_support"))]
    // assert!(counter_of_location == *grid.get_atomic_counter(&location).lock().unwrap(), "not the same");

    let mut index = 0;
    let mut buffer = String::from("counters of wolfs\n");
    for counter in &grid.values {
        let binding = counter.lock().unwrap();

        //print!("{:.2} ", *binding);
        let s = format!("{} ", *binding);
        buffer.push_str(&s);

        index = index + 1;
        if index == (DIM_X as i32 ) {
            buffer.push_str("\n");
            index = 0;
        }
    }
    println!("{}", buffer);

    // TEMP for debug purpose
}

fn update_sheeps_field(query_sheeps: Query<(Entity, &Sheep, &DBWrite<Location>)>, mut query_sheeps_field: Query<(&mut DenseBagGrid2D<Entity, SheepField>)>) {

    let mut sheeps_field = query_sheeps_field.single_mut();
    sheeps_field.clear();

    let process_sheep = |(entity, sheep, loc): (Entity, &Sheep, &DBWrite<Location>)| {
        if sheep.energy > 0. {
            sheeps_field.push_object_location(entity, &loc.0.0);
        }
    };

    query_sheeps.iter().for_each(process_sheep);

    // // TEMP for debug purpose
    // #[cfg(any(feature = "debug_support"))]
    // let mut non_empty_bags_counter = 0u32;

    // #[cfg(any(feature = "debug_support"))]
    // for bag in & sheeps_field.bags {
    //     if ! bag.is_empty() {
    //         non_empty_bags_counter += 1;
    //     }
    // }

    // #[cfg(any(feature = "debug_support"))]
    // println!("non empty bags: {}", non_empty_bags_counter);
    // // TEMP for debug purpose

    let mut index = 0;
    let mut buffer = String::from("counters of sheeps\n");
    for bag in &sheeps_field.bags {
        let s = format!("{} ", bag.len());
        buffer.push_str(&s);

        index = index + 1;
        if index == (DIM_X as i32 ) {
            buffer.push_str("\n");
            index = 0;
        }
    }
    println!("{}", buffer);
}

fn grass_grow(mut query_grass_field: Query<(&mut DenseSingleValueGrid2D<u16>)>) {
    // TODO Test if is good or not
    // TODO insert here some spans
    let mut grass_field = &mut query_grass_field.single_mut();

    grass_field.values.par_iter_mut().for_each(|grass_value| {
        let current_value = *grass_value;
        match(current_value) {
            Some(grass_value_u16) => {
                if grass_value_u16 < FULL_GROWN {
                    *grass_value = Some(grass_value_u16 + 1);
                }
            },
            None => {

            }
        }
    });
}

fn count_grass(grass_field: &DenseSingleValueGrid2D<u16>) -> i32 {
    let mut grass_growed = 0;
    grass_field.values.iter().for_each(|grass_value| {
        match(*grass_value) {
            Some(grass) => {
                if grass == FULL_GROWN {
                    grass_growed += 1;
                }
            }
            None => {

            }
        }
    });

    grass_growed
}





fn init_world(simulation_descriptor: Res<SimulationDescriptorT> ,mut commands: Commands) {

    #[cfg(any(feature = "debug_support"))]
    println!("init_world!");

    // T: generate the grass (START)
    #[cfg(any(feature = "debug_support"))]
    println!("generate grass");

    let mut grass_field = DenseSingleValueGrid2D::<u16>::new(DIM_X as i32, DIM_Y as i32);

    (0..DIM_X as i64).into_iter().for_each(|x| {
        (0..DIM_Y as i64).into_iter().for_each(|y| {

            #[cfg(not(any(feature = "fixed_random")))]
            let mut rng = rand::thread_rng();
            #[cfg(any(feature="fixed_random"))]
            let mut rng = RNG::new(simulation_descriptor.rand_seed, simulation_descriptor.current_step + (y * DIM_X as i64 + x) as u64 );


            let fully_growth = rng.gen_bool(0.5);
            if fully_growth {

                // T: TODO add the missing code with DenseGrid for Grass
                grass_field.set_value_location(FULL_GROWN, &Int2D { x: x.try_into().unwrap(), y: y.try_into().unwrap() });
            } else {
                let grass_init_value = rng.gen_range(0..FULL_GROWN + 1);

                grass_field.set_value_location(grass_init_value, &Int2D { x: x.try_into().unwrap(), y: y.try_into().unwrap() });
            }
        })
    });



    // TEMP for debug
    #[cfg(any(feature = "debug_support"))]
    let mut count = 0;

    #[cfg(any(feature = "debug_support"))]
    (0..DIM_X as i32).into_iter().for_each(|x| {
        (0..DIM_Y as i32).into_iter().for_each(|y| {

            //if state.grass_field.get_value(&Int2D {x, y}) == FULL_GROWN {
            //    count += 1;
            //}
            match grass_field.get_value(&Int2D {x, y}) {
                Some(grass) => {
                    if grass == FULL_GROWN {
                        count += 1;
                    }
                }
                None => {

                }
            }
        })
    });

    #[cfg(any(feature = "debug_support"))]
    println!("count: {}",count);
    // TEMP for debug

    commands.spawn((grass_field));
    // T: generate the grass (END)


    // T: generate sheeps (START)
    #[cfg(any(feature = "debug_support"))]
    println!("generate sheeps");

    for sheep_id in 0..NUM_INITIAL_SHEEPS {

        let id_to_assign = sheep_id + NUM_INITIAL_WOLFS;

        #[cfg(not(any(feature = "fixed_random")))]
        let mut rng = rand::thread_rng();
        #[cfg(any(feature="fixed_random"))]
        let mut rng = RNG::new(simulation_descriptor.rand_seed, simulation_descriptor.current_step + id_to_assign);

        let loc = Int2D { x: rng.gen_range(0..DIM_X as i32), y: rng.gen_range(0..DIM_Y as i32) };
        let initial_energy = rng.gen_range(0. ..(2. * GAIN_ENERGY_SHEEP));
        //println!("{}", initial_energy);

        let entity_commands = commands.spawn((
            Sheep {
                id: id_to_assign,
                energy: initial_energy,
            }, 

            DoubleBuffered::new(Location(loc)),
            DoubleBuffered::new(LastLocation(None)),

            Agent { id: id_to_assign + NUM_INITIAL_WOLFS },
        ));
    }

    let sheeps_field = DenseBagGrid2D::<Entity, SheepField>::new(DIM_X as i32, DIM_Y as i32);
    commands.spawn((sheeps_field));
    // T: generate sheeps (END)

    // T: generate wolfs (START)
    #[cfg(any(feature = "debug_support"))]
    println!("genereate wolfs");

    for wolf_id in 0..NUM_INITIAL_WOLFS {

        #[cfg(not(any(feature = "fixed_random")))]
        let mut rng = rand::thread_rng();
        #[cfg(any(feature="fixed_random"))]
        let mut rng = RNG::new(simulation_descriptor.rand_seed, simulation_descriptor.current_step + wolf_id);

        let loc = Int2D { x: rng.gen_range(0..DIM_X as i32), y: rng.gen_range(0..DIM_Y as i32) };
        let initial_energy = rng.gen_range(0. ..(2. * GAIN_ENERGY_WOLF));

        let entity_command = commands.spawn(
            
    (Wolf {
                id: wolf_id,
                energy: initial_energy,
            }, 

            DoubleBuffered::new(Location(loc)),
            DoubleBuffered::new(LastLocation(None)),

            Agent { id: wolf_id },
        ));
    }

    commands.spawn((AtomicGrid2D::<CountWolfs>::new(0u32, DIM_X as i32, DIM_Y as i32)));

    // let mut index = 0;
    // for counter in &mut counters_field.values {
    //     print!("({},{}): {}", index%counters_field.height, index/counters_field.height, counter.load(std::sync::atomic::Ordering::Acquire));

    //     index += 1;
    // }

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