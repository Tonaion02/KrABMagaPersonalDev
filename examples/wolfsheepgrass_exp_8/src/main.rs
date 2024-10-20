#![allow(warnings)]
//==============================================================================================================
//--------------------------------------------------------------------------------------------------------------
// WOLF-SHEEP-GRASS SIMULATION
// EXPERIMENT 8
//--------------------------------------------------------------------------------------------------------------
// LIST OF PARAMETERS OF SIMULATION
// STEPS: number of step the simulation must run
// NUM_AGENTS: The total number of agents the simulation starts with  
// NUM_THREADS: The total number of threads the simulation can use. This parameter is considered only if 
// we pass the correct command line features to the compiler.
// PERC_WOLF: The percentage of wolves about NUM_AGENTS in this simulation.
// PERC_SHEEPS: The percentage of sheep about NUM_AGENTS in this simulation.
// DIM_X, DIM_Y: Dimension of the enviroment, correspond to the number of possible position that the agents
// can be and the number of cells a grid is formed.
//--------------------------------------------------------------------------------------------------------------
// Read to understand better the code base
// The comment's lines that start with 'T:' are left by Tonaion02.
// The comment's lines where there is 'TODO' is a reminder for something that we must do.
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

use engine::resources::cimitery_buffer_exp_7::CimiteryBufferExp7;
// T: importing lazy_static
use lazy_static::lazy_static;
use model::debug::info_debug;

use std::borrow::Borrow;
use std::env::consts::EXE_SUFFIX;
use std::marker::PhantomData;
use std::time::Instant;

use std::hash::Hash;

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;

use std::cell::RefCell;

use std::thread::LocalKey;

use std::cmp::min;





// T: model's import (START)
mod model;
use crate::model::animals::Sheep;
use crate::model::animals::Wolf;
use crate::model::animals::Location;
use crate::model::animals::LastLocation;
use crate::model::animals::Agent;
// T: model's import (END)

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

use engine::thread_id::thread_id;

#[cfg(not(any(feature="fixed_random")))]
use krabmaga::rand::Rng;

use engine::location::Real2D;
use engine::location::Int2D;
use krabmaga::engine::simulation::Simulation;
use krabmaga::engine::fields::dense_number_grid_2d_t::DenseSingleValueGrid2D;
use krabmaga::engine::fields::parallel_dense_object_grid_2d_exp_6::ParDenseBagGrid2D_exp_6;

use krabmaga::engine::resources::simulation_descriptor::SimulationDescriptorT;

use krabmaga::engine::simulation::SimulationSet::Step;
use krabmaga::engine::simulation::SimulationSet::AfterStep;
use krabmaga::engine::simulation::SimulationSet::BeforeStep;

// T: TODO verify if it is useless
use krabmaga::engine::rng::RNG;
use krabmaga::engine::SampleRange;

// T: bevy's import (START)
// T: TODO find a way to remove the necessity to use these
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
use krabmaga::engine::QueryState;

use krabmaga::engine::bevy_utils::Parallel;
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
pub const SEED: u64 = 21382193872;

pub const SIMULATION_TITLE: &'static str = "WolfSheepGrass_exp_8";

// MODIFIED: now we retrieve this parameters from command line
// but that acts like "static constants", little trick from here:
// https://stackoverflow.com/questions/37405835/populating-a-static-const-with-an-environment-variable-at-runtime-in-rust
lazy_static! {
    static ref NUM_THREADS: usize = 
    match (std::env::args().collect::<Vec<String>>().get(1)) {
        Some(value) => { value.clone().parse::<usize>().unwrap() }
        None => { 0usize }
    };

    static ref NUM_AGENTS: u64 = 
    match (std::env::args().collect::<Vec<String>>().get(2)) {
        Some(value) => { value.clone().parse::<u64>().unwrap() }
        None => { 0u64 }
    };

    static ref DIM_X: f64 = 
    match (std::env::args().collect::<Vec<String>>().get(3)) {
        Some(value) => { value.clone().parse::<f64>().unwrap() }
        None => { 0. }
    };

    static ref STEPS: u32 =
    match (std::env::args().collect::<Vec<String>>().get(4)) {
        Some(value) => { value.clone().parse::<u32>().unwrap() }
        None => { 0u32 }
    };

    static ref DIM_Y: f64 = *DIM_X;

    static ref PERC_SHEEPS: f64 = 0.6;
    static ref PERC_WOLFS: f64 = 0.4;
    
    static ref NUM_INITIAL_SHEEPS: u64 = ((*NUM_AGENTS as f64) * (*PERC_SHEEPS)) as u64;
    static ref NUM_INITIAL_WOLFS: u64 = ((*NUM_AGENTS as f64) * (*PERC_WOLFS)) as u64;
}
// T: new costants(END)

// T: Constants(END)





// T: markers for fields (START)
pub struct SheepField;
pub struct WolfField;

pub struct WolvesBuffer;
pub struct SheepBuffer;
pub struct DeletedWolvesBuffer;
pub struct DeletedSheepBuffer;
// T: markers for fields (END)

// T: create structure to keep buffers (START)
#[derive(Component)]
pub struct AgentBuffer<O: Send,M: Sized> {
    pub internal_buffer: Parallel::<Vec<O>>, 

    phantom: PhantomData<M>,
}

impl<O: Send, M: Sized> AgentBuffer<O, M> {

    pub fn new() -> AgentBuffer<O, M> {
        AgentBuffer {
            internal_buffer: Parallel::<Vec<O>>::default(),

            phantom: PhantomData,
        }
    }

}
// T: create structure to keep buffers (END)





// 'No-visualization' specific imports
#[cfg(not(any(feature = "visualization", feature = "visualization_wasm")))]
use krabmaga::*;

#[cfg(not(any(feature = "visualization", feature = "visualization_wasm")))]
fn main() {

    let now = Instant::now();

    let simulation = build_simulation();
    simulation.run();

    let elapsed_time = now.elapsed();
    println!("Elapsed: {:.2?}, steps per second: {}", elapsed_time, *STEPS as f64 / elapsed_time.as_secs_f64());
    save_elapsed_time(elapsed_time);
}

fn build_simulation() -> Simulation {

    // T: Setting rayon's enviroment variable (START)
    // T: TODO move this in the correct place

    rayon::ThreadPoolBuilder::new().
    num_threads(*NUM_THREADS).

    start_handler(|real_thread_id| {
        thread_id.with(|cell| { cell.set(real_thread_id); });
    }).
    build_global().
    unwrap();
    // T: Setting rayon's enviroment variable (END)





    let mut simulation = Simulation::build();
    simulation = simulation
    .with_title(String::from(SIMULATION_TITLE))
    .with_steps(*STEPS)
    .with_num_threads(*NUM_THREADS)
    .with_simulation_dim(Real2D {x: *DIM_X as f32, y: *DIM_Y as f32}) 
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

    // T: Must run after the despawning of entities
    app.add_systems(Update, update_wolves_field.in_set(BeforeStep));
    app.add_systems(Update, update_sheeps_field.in_set(BeforeStep));
    app.add_systems(Update, grass_grow.in_set(BeforeStep));

    // T: added to recycle entities
    app.add_systems(Update, cimitery_system.in_set(AfterStep));
    
    // app.add_systems(Update, info_debug.in_set(AfterStep).after(cimitery_system));
    // app.add_systems(Update, count_agents.in_set(BeforeStep));
    // app.add_systems(Update, count_sheeps.in_set(BeforeStep));
    // app.add_systems(Update, count_wolfs.in_set(BeforeStep));

    #[cfg(any(feature = "debug_support"))]
    app.add_systems(Update, population_debug_info.in_set(BeforeStep).before(update_wolves_field).before(update_sheeps_field).before(grass_grow));
    #[cfg(any(feature = "debug_support"))]
    app.add_systems(Update, print_step.in_set(BeforeStep).before(population_debug_info).before(update_wolves_field).before(update_sheeps_field).before(grass_grow));

    simulation
}





// Unique step function
fn step ( 
    mut query_grass_field: Query<&mut DenseSingleValueGrid2D<u16>>,
    mut query_sheeps: Query<(Entity, &mut Sheep, &DBRead<Location>)>,

    mut query_agents: Query<(&Agent, &mut DBWrite<Location>, &mut DBWrite<LastLocation>)>,

    query_wolfs_field: Query<(&ParDenseBagGrid2D_exp_6<Entity, WolfField>)>,
    query_sheeps_field: Query<(&ParDenseBagGrid2D_exp_6<Entity, SheepField>)>,

    query_wolfs: Query<(Entity, &Wolf, &DBRead<Location>)>,

    mut parallel_commands: ParallelCommands,

    mut query_wolves_buffer: Query<(&mut AgentBuffer<(Wolf, DoubleBuffered<Location>, DoubleBuffered<LastLocation>, Agent), WolvesBuffer>)>,
    mut query_sheep_buffer: Query<(&mut AgentBuffer<(Sheep, DoubleBuffered<Location>, DoubleBuffered<LastLocation>, Agent), SheepBuffer>)>,
    mut query_deleted_wolves_buffer: Query<(&mut AgentBuffer<(Entity), DeletedWolvesBuffer>)>,
    mut query_deleted_sheep_buffer: Query<(&mut AgentBuffer<(Entity), DeletedSheepBuffer>)>,

    simulation_descriptor: Res<SimulationDescriptorT>,
)
{
    // T: Retrieve buffers for each category of agents (START)
    let wolves_buffer = query_wolves_buffer.single_mut();
    let sheep_buffer = query_sheep_buffer.single_mut();

    let deleted_wolves_buffer = query_deleted_wolves_buffer.single_mut();
    let deleted_sheep_buffer = query_deleted_sheep_buffer.single_mut();
    // T: Retrieve buffers for each category of agents (END)

    let mut grass_field = query_grass_field.single_mut();



    // T: move agents (START)
    let span = info_span!("move agents");
    let span = span.enter();

    query_agents.par_iter_mut().for_each(|(agent, mut loc, mut last_loc)|{

        let x = loc.0.0.x;
        let y = loc.0.0.y;

        let mut moved = false;

        #[cfg(any(feature="fixed_random"))]
        let mut rng = RNG::new(simulation_descriptor.rand_seed, simulation_descriptor.current_step);
        #[cfg(not(any(feature="fixed_random")))]
        let mut rng = rand::thread_rng();

        if last_loc.0.0.is_some() && rng.gen_bool(MOMENTUM_PROBABILITY as f64) {
            if let Some(pos) = last_loc.0.0 {
                let xm = x + (x - pos.x);
                let ym = y + (y - pos.y);
                let new_loc = Int2D { x: xm, y: ym };
                // TRY TO MOVE WITH MOMENTUM_PROBABILITY
                if xm >= 0 && xm < *DIM_X as i32 && ym >= 0 && ym < *DIM_Y as i32 {
                    loc.0 = Location(new_loc);
                    last_loc.0 = LastLocation(Some(Int2D { x, y }));
                    moved = true;
                }
            }
        }

        if !moved {
            let xmin = if x > 0 { -1 } else { 0 };
            let xmax = i32::from(x < *DIM_X as i32 - 1);
            let ymin = if y > 0 { -1 } else { 0 };
            let ymax = i32::from(y < *DIM_Y as i32 - 1);

            // let nx = if rng.gen_bool(0.5) { xmin } else { xmax };
            // let ny = if rng.gen_bool(0.5) { ymin } else { ymax };
            let nx = rng.gen_range(xmin..=xmax);
            let ny = rng.gen_range(ymin..=ymax);

            loc.0 = Location(Int2D { x: x + nx, y: y + ny, });
            last_loc.0 = LastLocation(Some(Int2D { x, y }));
        }
    });

    std::mem::drop(span);
    // T: move agents (END)



    // T: Sheeps eat (START)
    let span = info_span!("sheeps eats");
    let span = span.enter();

    query_sheeps.iter_mut().for_each(|(entity, mut sheep_data, sheep_loc)|{
        if grass_field.get_value(&sheep_loc.0.0).expect("empty cell(not possible!)") >= FULL_GROWN {
            grass_field.set_value_location(0, &sheep_loc.0.0);
            
            sheep_data.energy += GAIN_ENERGY_SHEEP;
        }
    });

    std::mem::drop(span);
    // T: Sheeps eat (END)



    // T: Sheeps reproduce (START)
    let span = info_span!("sheeps reproduce");
    let span = span.enter();

    // T: parallel version
    query_sheeps.par_iter_mut().for_each(|(entity, mut sheep_data, loc)| {

        sheep_data.energy -= ENERGY_CONSUME;

        
        #[cfg(not(any(feature="fixed_random")))]
        let mut rng = rand::thread_rng();
        #[cfg(any(feature="fixed_random"))]
        let mut rng = RNG::new(simulation_descriptor.rand_seed, simulation_descriptor.current_step);

            if sheep_data.energy > 0. && rng.gen_bool(SHEEP_REPR as f64) {
    
                sheep_data.energy /= 2.0;

                sheep_buffer.internal_buffer.scope(|coll| {
                    coll.push((
                            Sheep {
                                id: 0,
                                energy: sheep_data.energy,
                            },
        
                            DoubleBuffered::new(Location(loc.0.0)),
                            DoubleBuffered::new(LastLocation(None)),
        
                            Agent {id: 0},
                        ));
                    });
                    
                }
                if sheep_data.energy <= 0. {
                    deleted_sheep_buffer.internal_buffer.scope(|(coll)| {
                        coll.push(entity);
                    });                    
                }
    });

    std::mem::drop(span);
    // T: Sheeps reproduce (END)



    // T: Wolves eat (START)
    let span = info_span!("wolves eat");
    let span = span.enter();

    let sheep_field = query_sheeps_field.single();
    let wolfs_field = query_wolfs_field.single();

    let sheep_field_iter = sheep_field.bags.par_iter();
    wolfs_field.bags.par_iter().zip(sheep_field_iter).for_each(|(wolf_bag_lock, sheep_bag_lock)| {

        // Locking resources to make parallelization (START) 
        let wolf_bag = wolf_bag_lock.read().unwrap();
        let sheep_bag = sheep_bag_lock.read().unwrap();
        // Locking resources to make parallelization (END)

        let mut wolf_index = 0usize;
        let mut sheep_index = 0usize;

        // T: for all the wolves in the bag (START)
        while(wolf_index < wolf_bag.len()) {

            // T: Search an alive sheep in the bag of sheep (START)
            while(sheep_index < sheep_bag.len()) {

                let sheep_entity = sheep_bag[sheep_index];
                let sheep_data = query_sheeps.get(sheep_entity).unwrap().1;

                sheep_index += 1;

                if sheep_data.energy > 0. {

                    deleted_sheep_buffer.internal_buffer.scope(|(coll)| {
                        coll.push(sheep_entity);
                    });

                    let wolf_entity = wolf_bag[wolf_index];
                    let wolf_data = query_wolfs.get(wolf_entity).unwrap().1;
                    let mut energy_wolf = wolf_data.energy.lock().unwrap();
                    *energy_wolf += GAIN_ENERGY_WOLF;

                    break;
                }
            }
            // T: Search an alive sheep in the bag of sheep (END)

            wolf_index += 1;
        }
        // T: for all the wolves in the bag (END)
    });

    std::mem::drop(span);
    // T: Wolves eat (END)



    // T: Reproduce wolves (START)
    let span = info_span!("reproducing wolves");
    let span = span.enter();



    // T: Parallel version
    query_wolfs.par_iter().for_each(
        |(entity, wolf_data, loc)| {

            let mut energy_wolf = wolf_data.energy.lock().unwrap();
            *energy_wolf -= ENERGY_CONSUME;

            
                      
            #[cfg(not(any(feature="fixed_random")))]
            let mut rng_div = rand::thread_rng(); 
            #[cfg(any(feature="fixed_random"))]
            let mut rng_div = RNG::new(simulation_descriptor.rand_seed, simulation_descriptor.current_step);
                
            if *energy_wolf > 0. && rng_div.gen_bool(WOLF_REPR as f64) {
                *energy_wolf /= 2.0;
                                
                wolves_buffer.internal_buffer.scope(|(coll)| {
                    coll.push((
                        Wolf {
                            id: 0,
                            energy: Mutex::new(*energy_wolf),
                            }, 
                  
                        DoubleBuffered::new(Location(loc.0.0)),
                        DoubleBuffered::new(LastLocation(None)),
                  
                        Agent {id: 0,},));
                });

            }
            if *energy_wolf <= 0. {
                deleted_wolves_buffer.internal_buffer.scope(|(coll)| {
                    coll.push(entity);
                });
            }              
            
        }
    );
    
    std::mem::drop(span);
    // T: Reproduce wolves (END)
}





// T: Cimitery system (START)
// T: TODO evaluate how this method can be abstracted away
// T: probably the user must define only the archetype that reppresent a Type
// T: of agent in the simulation, and then the system can recycle existing
// T: agents to spawn the new agents
// T: NOTES:
// T: we must consider two different situations:
// T:   - we don't have enough dead entities to cover the entities that we
// T:     want to spawn.
// T:   - we have more deleted entities than entities to spawn.
// T: For the first, we can simple spawn a batch of new agents with spawn_batch
// T: hoping that this can be really 
// T: TODO consider to not re-allocate each time a new buffer for the agents.
fn cimitery_system(
    world: &mut World,

    mut query_wolves: &mut QueryState<(&mut Wolf, &mut DBWrite<Location>, &mut DBRead<Location>, &mut DBWrite<LastLocation>, &mut DBRead<LastLocation>, &mut Agent)>,
    mut query_wolves_buffer: &mut QueryState<(&mut AgentBuffer<(Wolf, DoubleBuffered<Location>, DoubleBuffered<LastLocation>, Agent), WolvesBuffer>)>,
    mut query_deleted_wolves_buffer: &mut QueryState<(&mut AgentBuffer<(Entity), DeletedWolvesBuffer>)>,
    
    mut query_sheep: &mut QueryState<(&mut Sheep, &mut DBWrite<Location>, &mut DBRead<Location>, &mut DBWrite<LastLocation>, &mut DBRead<LastLocation>, &mut Agent)>,
    mut query_sheep_buffer: &mut QueryState<(&mut AgentBuffer<(Sheep, DoubleBuffered<Location>, DoubleBuffered<LastLocation>, Agent), SheepBuffer>)>,
    mut query_deleted_sheep_buffer: &mut QueryState<(&mut AgentBuffer<(Entity), DeletedSheepBuffer>)>,
) {
    // T: DEBUG
    // println!("Recycling entities!");



    let recycle_entities_span = info_span!("Recycle entities");
    let recycle_entities_span = recycle_entities_span.enter();



    // T: Recycle entities from pool of death entities for WOLVES (START)
    let span = info_span!("Recycling entities of wolves");
    let span = span.enter();


    // T: Retrieving elements from parallels buffers (START)
    let mut wolves_buffer = query_wolves_buffer.single_mut(world);
    let mut wolves_buffer_vec = Vec::<(Wolf, DoubleBuffered<Location>, DoubleBuffered<LastLocation>, Agent)>::new();
    wolves_buffer.internal_buffer.drain_into(&mut wolves_buffer_vec);
    std::mem::drop(wolves_buffer);

    let mut deleted_wolves_buffer = query_deleted_wolves_buffer.single_mut(world);
    let mut deleted_wolves_buffer_vec = Vec::<(Entity)>::new();
    deleted_wolves_buffer.internal_buffer.drain_into(&mut deleted_wolves_buffer_vec);
    std::mem::drop(deleted_wolves_buffer);
    // T: Retrieving elements from parallels buffers (END)


    
    // T: compute minimum between size of buffers
    let min_wolves_number = min(wolves_buffer_vec.len(), deleted_wolves_buffer_vec.len());

    let span_effectively_recycling = info_span!("Effectively recycling wolves");
    let span_effectively_recycling = span_effectively_recycling.enter();



    if(wolves_buffer_vec.len() > 0 && deleted_wolves_buffer_vec.len() > 0) {
        // T: DEBUG
        // println!("Entered in wolves if");
    let mut slice_for_wolves = &mut wolves_buffer_vec[..min_wolves_number];
    let mut slice_for_deleted_wolves = &mut deleted_wolves_buffer_vec[..min_wolves_number];

    // T: Retrieve slices on the base of mimimum size buffers
    let mut iter_mut_slice_for_wolves = slice_for_wolves.iter_mut();
    let mut iter_mut_slice_for_deleted_wolves = slice_for_deleted_wolves.iter_mut();
    
    // T: Iterate on couple formed by new_wolf and deleted_wolf_entity (START)
    // T: new_wolf is an n-uple formed by all the data necessary to initialize a new wolf
    // T: deleted_wolf is an Entity that indicates an Entity that has died and can be re-used 
    iter_mut_slice_for_wolves.zip(iter_mut_slice_for_deleted_wolves).for_each(|(new_wolf, deleted_wolf_entity)|{
        let mut deleted_wolf = query_wolves.get_mut(world, *deleted_wolf_entity).expect("Is not possible!");
        
        // T: NOTES very problematic part to search to automize this part of the code (START)
        // T: It's difficult to write a generic copy for the data
        *deleted_wolf.0.energy.lock().unwrap() = *new_wolf.0.energy.lock().unwrap();
        deleted_wolf.0.id = new_wolf.0.id;
        *deleted_wolf.1 = new_wolf.1.write; 
        *deleted_wolf.2 = new_wolf.1.read;
        *deleted_wolf.3 = new_wolf.2.write;
        *deleted_wolf.4 = new_wolf.2.read;
        *deleted_wolf.5 = new_wolf.3;
        // T: NOTES very problematic part to search to automize this part of the code (END)
    });
    // T: Iterate on couple formed by new_wolf and deleted_wolf_entity (END)
    }    

    std::mem::drop(span_effectively_recycling);



    // T: Handle the two problematic case (START)

    // T: Case where the dead agents is less than the number of agents to spawn for this type (START)
    // T: NOTES: there is the inefficiency to re-allocate some memory during this operation
    // T: I cannot find a way to pass a slice to spawn_batch
    if (wolves_buffer_vec.len() > deleted_wolves_buffer_vec.len()) {
        let span_spawning_remaining_agents = info_span!("spawning remaining agents(WOLFS)");
        let span_spawning_remaining_agents = span_spawning_remaining_agents.enter();     

        let remaining_wolves_to_spawn: Vec::<(Wolf, DoubleBuffered<Location>, DoubleBuffered<LastLocation>, Agent)> = wolves_buffer_vec.drain(min_wolves_number+1..).collect();
        world.spawn_batch(remaining_wolves_to_spawn);

        std::mem::drop(span_spawning_remaining_agents);
    }
    // T: Case where the dead agents is less than the number of agents to spawn for this type (END)
    
    // T: Case where the dead agents is more than the number of agents to spawn for this type (START)
    else if(wolves_buffer_vec.len() < deleted_wolves_buffer_vec.len()) {
        let span_despawn_remaining_agents = info_span!("despawning remaining agents(WOLFS)");
        let span_despawn_remaining_agents = span_despawn_remaining_agents.enter();

        let remaining_slice_for_deleted_wolves = &deleted_wolves_buffer_vec[min_wolves_number+1..];
        remaining_slice_for_deleted_wolves.iter().for_each(|(deleted_wolf_entity)| {
            world.despawn(*deleted_wolf_entity);
        });

        std::mem::drop(span_despawn_remaining_agents);
    }
    // T: Case where the dead agents is more than the number of agents to spawn for this type (END)

    // T: Handle the two problematic case (END)


    
    std::mem::drop(span);
    // T: Recycle entities from pool of death entities for WOLVES (END)





    // T: Recycle entities from pool of death entities for SHEEP (START)
    let span = info_span!("Recycling entities of sheep");
    let span = span.enter();



    let mut sheep_buffer = query_sheep_buffer.single_mut(world);
    let mut sheep_buffer_vec = Vec::<(Sheep, DoubleBuffered<Location>, DoubleBuffered<LastLocation>, Agent)>::new();
    sheep_buffer.internal_buffer.drain_into(&mut sheep_buffer_vec);
    std::mem::drop(sheep_buffer);

    let mut deleted_sheep_buffer = query_deleted_sheep_buffer.single_mut(world);    
    let mut deleted_sheep_buffer_vec = Vec::<(Entity)>::new();
    deleted_sheep_buffer.internal_buffer.drain_into(&mut deleted_sheep_buffer_vec);
    std::mem::drop(deleted_sheep_buffer);

    // T: compute minimum between size of buffers
    let min_sheep_number = min(sheep_buffer_vec.len(), deleted_sheep_buffer_vec.len());

    let span_effectively_recycling = info_span!("Effectively recycling sheep");
    let span_effectively_recycling = span_effectively_recycling.enter();



    if(sheep_buffer_vec.len() > 0 && deleted_sheep_buffer_vec.len() > 0) {
        // T: DEBUG
        // println!("Entered in sheep if");
    let mut slice_for_sheep = &mut sheep_buffer_vec[..min_sheep_number];
    let mut slice_for_deleted_sheep = &mut deleted_sheep_buffer_vec[..min_sheep_number];

    // T: Retrieve slices on the base of mimimum size buffers
    let mut iter_mut_slice_for_sheep = slice_for_sheep.iter_mut();
    let mut iter_mut_slice_for_deleted_sheep = slice_for_deleted_sheep.iter_mut();

    // T: Iterate on couple formed by new_sheep and deleted_sheep_entity (START)
    // T: new_sheep is an n-uple formed by all the data necessary to initialize a new sheep
    // T: deleted_sheep is an Entity that indicates an Entity that has died and can be re-used 
    iter_mut_slice_for_sheep.zip(iter_mut_slice_for_deleted_sheep).for_each(|(new_sheep, deleted_sheep_entity)|{
        let mut deleted_sheep = query_sheep.get_mut(world, *deleted_sheep_entity).expect("Is not possible!");
        
        *deleted_sheep.0 = new_sheep.0;
        *deleted_sheep.1 = new_sheep.1.write;
        *deleted_sheep.2 = new_sheep.1.read;
        *deleted_sheep.3 = new_sheep.2.write;
        *deleted_sheep.4 = new_sheep.2.read;
        *deleted_sheep.5 = new_sheep.3;
    });
    // T: Iterate on couple formed by new_sheep and deleted_sheep_entity (END)
    }

    std::mem::drop(span_effectively_recycling);



    // T: Handle the two problematic case (START)

    // T: Case where the dead agents is less than the number of agents to spawn for this type (START)
    // T: NOTES: there is the inefficiency to re-allocate some memory during this operation
    // T: I cannot find a way to pass a slice to spawn_batch
    
    if (sheep_buffer_vec.len() > deleted_sheep_buffer_vec.len()) {
        let span_spawning_remaining_agents = info_span!("spawning remaining agents(SHEEP)");
        let span_spawning_remaining_agents = span_spawning_remaining_agents.enter();     

        let remaining_sheep_to_spawn: Vec::<(Sheep, DoubleBuffered<Location>, DoubleBuffered<LastLocation>, Agent)> = sheep_buffer_vec.drain(min_sheep_number+1..).collect();
        world.spawn_batch(remaining_sheep_to_spawn);

        std::mem::drop(span_spawning_remaining_agents);
    }
    // T: Case where the dead agents is less than the number of agents to spawn for this type (END)
    
    // T: Case where the dead agents is more than the number of agents to spawn for this type (START)
    else if(sheep_buffer_vec.len() < deleted_sheep_buffer_vec.len()) {
        let span_despawn_remaining_agents = info_span!("despawning remaining agents(SHEEP)");
        let span_despawn_remaining_agents = span_despawn_remaining_agents.enter();

        let remaining_slice_for_deleted_sheep = &deleted_sheep_buffer_vec[min_sheep_number+1..];
        remaining_slice_for_deleted_sheep.iter().for_each(|(deleted_sheep_entity)| {
            world.despawn(*deleted_sheep_entity);
        });

        std::mem::drop(span_despawn_remaining_agents);
    }
    // T: Case where the dead agents is more than the number of agents to spawn for this type (END)

    // T: Handle the two problematic case (END)



    std::mem::drop(span);
    // T: Recycle entities from pool of death entities for SHEEP (END)



    std::mem::drop(recycle_entities_span);
}
// T: Cimitery system (END)





// T: Run before step (START)
fn update_wolves_field(
    query_wolfs: Query<(Entity, &Wolf, &DBWrite<Location>)>, 
    mut query_wolfs_field: Query<(&mut ParDenseBagGrid2D_exp_6<Entity, WolfField>)>,
) {

    let mut wolfs_field = query_wolfs_field.single_mut();
    wolfs_field.clear();

    let process_wolf = |(entity, wolf, loc) : (Entity, &Wolf, &DBWrite<Location>)| {

        let mut wolf_bag = wolfs_field.get_write_bag(&loc.0.0);
        wolf_bag.push(entity);
    };

    query_wolfs.par_iter().for_each(process_wolf);
}

fn update_sheeps_field(
    query_sheeps: Query<(Entity, &Sheep, &DBWrite<Location>)>, 
    mut query_sheeps_field: Query<(&mut ParDenseBagGrid2D_exp_6<Entity, SheepField>)>,
) {

    let mut sheeps_field = query_sheeps_field.single_mut();
    sheeps_field.clear();

    let process_sheep = |(entity, sheep, loc): (Entity, &Sheep, &DBWrite<Location>)| {

        let mut sheep_bag = sheeps_field.get_write_bag(&loc.0.0);
        sheep_bag.push(entity);
    };

    query_sheeps.par_iter().for_each(process_sheep);
}

fn grass_grow(mut query_grass_field: Query<(&mut DenseSingleValueGrid2D<u16>)>) {
    // TODO Test if it is good or not for performances
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
// T: Run before step (END)

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

    let mut grass_field = DenseSingleValueGrid2D::<u16>::new(*DIM_X as i32, *DIM_Y as i32);

    (0.. *DIM_X as i64).into_iter().for_each(|x| {
        (0.. *DIM_Y as i64).into_iter().for_each(|y| {

            #[cfg(not(any(feature = "fixed_random")))]
            let mut rng = rand::thread_rng();
            #[cfg(any(feature="fixed_random"))]
            let mut rng = RNG::new(simulation_descriptor.rand_seed, simulation_descriptor.current_step + (y * DIM_X as i64 + x) as u64 );


            let fully_growth = rng.gen_bool(0.5);
            if fully_growth {

                // T: TODO add the missing code with DenseGrid for Grass
                grass_field.set_value_location(FULL_GROWN, &Int2D { x: x.try_into().unwrap(), y: y.try_into().unwrap() });
            } else {
                // T: original version that doesn't fit really well with the example
                // let grass_init_value = rng.gen_range(0..FULL_GROWN + 1);
                // grass_field.set_value_location(grass_init_value, &Int2D { x: x.try_into().unwrap(), y: y.try_into().unwrap() });

                grass_field.set_value_location(0, &Int2D { x: x.try_into().unwrap(), y: y.try_into().unwrap() });
            }
        })
    });

    commands.spawn((grass_field));
    // T: generate the grass (END)



    // T: generate sheep (START)
    #[cfg(any(feature = "debug_support"))]
    println!("generate sheeps");

    for sheep_id in 0.. *NUM_INITIAL_SHEEPS {

        let id_to_assign = sheep_id + *NUM_INITIAL_WOLFS;

        #[cfg(not(any(feature = "fixed_random")))]
        let mut rng = rand::thread_rng();
        #[cfg(any(feature="fixed_random"))]
        let mut rng = RNG::new(simulation_descriptor.rand_seed, simulation_descriptor.current_step + id_to_assign);

        let loc = Int2D { x: rng.gen_range(0.. *DIM_X as i32), y: rng.gen_range(0.. *DIM_Y as i32) };
        let initial_energy = rng.gen_range(0. ..(2. * GAIN_ENERGY_SHEEP));
        //println!("{}", initial_energy);

        let entity_commands = commands.spawn((
            Sheep {
                id: id_to_assign,
                energy: initial_energy,
            }, 

            DoubleBuffered::new(Location(loc)),
            DoubleBuffered::new(LastLocation(None)),

            Agent { id: id_to_assign + *NUM_INITIAL_WOLFS },
        ));
    }

    let sheep_field = ParDenseBagGrid2D_exp_6::<Entity, SheepField>::new(*DIM_X as i32, *DIM_Y as i32);
    commands.spawn((sheep_field));
    // T: generate sheep (END)



    // T: generate wolves (START)
    #[cfg(any(feature = "debug_support"))]
    println!("genereate wolfs");

    for wolf_id in 0.. *NUM_INITIAL_WOLFS {

        #[cfg(not(any(feature = "fixed_random")))]
        let mut rng = rand::thread_rng();
        #[cfg(any(feature="fixed_random"))]
        let mut rng = RNG::new(simulation_descriptor.rand_seed, simulation_descriptor.current_step + wolf_id);

        let loc = Int2D { x: rng.gen_range(0.. *DIM_X as i32), y: rng.gen_range(0.. *DIM_Y as i32) };
        let initial_energy = rng.gen_range(0. ..(2. * GAIN_ENERGY_WOLF));

        let entity_command = commands.spawn(
            
    (Wolf {
                id: wolf_id,
                energy: Mutex::new(initial_energy),
            }, 

            DoubleBuffered::new(Location(loc)),
            DoubleBuffered::new(LastLocation(None)),

            Agent { id: wolf_id },
        ));
    }

    let wolves_field = ParDenseBagGrid2D_exp_6::<Entity, WolfField>::new(*DIM_X as i32, *DIM_Y as i32);
    commands.spawn((wolves_field));
    // T: generate wolves (END)




    // T: Initialize buffers for CimiterySystem (START)
    // T: TODO replace this definitions with some types of macros that can save
    // T: some orrible syntax to people. 
    // T: I want to keep all the orrible syntax only for me.
    let mut wolvesBuffer = AgentBuffer::<(Wolf, DoubleBuffered<Location>, DoubleBuffered<LastLocation>, Agent), WolvesBuffer>::new();
    commands.spawn((wolvesBuffer));

    let mut sheepBuffer = AgentBuffer::<(Sheep, DoubleBuffered<Location>, DoubleBuffered<LastLocation>, Agent), SheepBuffer>::new();
    commands.spawn((sheepBuffer));
    
    // T: Added to save the entities to delete
    let mut deletedWolvesBuffer = AgentBuffer::<(Entity), DeletedWolvesBuffer>::new();
    commands.spawn((deletedWolvesBuffer));

    let mut deletedSheepBuffer = AgentBuffer::<(Entity), DeletedSheepBuffer>::new();
    commands.spawn((deletedSheepBuffer));
    // T: Initialize buffers for CimiterySystem (END)
}





// T: TODO check what macro make this work before ECS experiment
fn save_elapsed_time(elapsed_time: core::time::Duration) {
    
    use std::path::Path;
    use std::fs::File;
    use std::io::prelude::*;
    
    //Write on file the elapsed time
    let path = Path::new("elapsed_time.txt");
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