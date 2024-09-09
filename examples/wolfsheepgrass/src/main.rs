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

// T: importing rayon (START)
extern crate rayon;
use crate::rayon::iter::IntoParallelRefIterator;
use crate::rayon::iter::IntoParallelRefMutIterator;
use crate::rayon::iter::ParallelIterator;
use crate::rayon::iter::IndexedParallelIterator;
// T: importing rayon (END)

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

use krabmaga::engine::simulation::SimulationSet::Step;
use krabmaga::engine::simulation::SimulationSet::AfterStep;
use krabmaga::engine::simulation::SimulationSet::BeforeStep;

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
    //app.add_systems(Update, count_wolfs_for_location);
    app.add_systems(Update, move_agents.in_set(Step));
    app.add_systems(Update, sheeps_eat.in_set(Step).after(move_agents));
    app.add_systems(Update, sheeps_die.in_set(Step).after(sheeps_eat));
    app.add_systems(Update, wolfs_eat.in_set(Step).after(sheeps_die));
    app.add_systems(Update, reproduce_wolves.in_set(Step).after(wolfs_eat));
    app.add_systems(Update, reproduce_sheeps.in_set(Step).after(reproduce_wolves));
    // Must run after the despawning of entities
    app.add_systems(Update, count_wolfs_for_location.in_set(BeforeStep));
    app.add_systems(Update, update_sheeps_field.in_set(BeforeStep));
    app.add_systems(Update, grass_grow.in_set(BeforeStep));

    // T: TEMP
    // T: only for debug purpose
    //app.add_systems(Update, count_agents);
    
    //app.add_systems(Update, count_sheeps);
    //app.add_systems(Update, count_wolfs);

    simulation
}





// Run at the start, like an update of the fields.......(hoping that can work)
fn count_wolfs_for_location(query_wolfs: Query<(&Wolf, &DBWrite<Location>)>, mut query_count_grid: Query<(&AtomicGrid2D<CountWolfs>)>) {

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

// T: This method must run after sheeps_die
fn wolfs_eat(mut query_wolfs: Query<(&mut Wolf, &DBRead<Location>)>, 
            mut query_count_grid: Query<(&AtomicGrid2D<CountWolfs>)>,
            query_sheeps_field: Query<(&DenseBagGrid2D<Entity, SheepField>)> ) {

    let mut grid = query_count_grid.single_mut();
    let sheep_field = query_sheeps_field.single();

    query_wolfs.par_iter_mut().for_each(|(mut wolf_data, wolf_loc)| {
        let binding = grid.get_atomic_counter(&wolf_loc.0.0);
        let mut counter = binding.lock().unwrap();
        let sheeps_for_bag = sheep_field.get_ref_bag(&wolf_loc.0.0).len();
        if counter.0 > sheeps_for_bag as u32 {
            counter.0 = sheeps_for_bag as u32;
        }
        counter.0 -= 1;
        std::mem::drop(counter);

        wolf_data.energy += GAIN_ENERGY_WOLF;
    });
}

// T: This method must run before wolf_eat
fn sheeps_die(query_sheeps_field: Query<(&DenseBagGrid2D<Entity, SheepField>)>,
            mut query_count_grid: Query<(&AtomicGrid2D<CountWolfs>)>,
            mut parallel_commands: ParallelCommands) {

    let mut grid = query_count_grid.single_mut();
    let sheeps_field = query_sheeps_field.single();
    
    let grid_par_iterator = grid.values.par_iter();
    sheeps_field.bags.par_iter().zip(grid_par_iterator).for_each(|(bag, binding)|{
        let counter = binding.lock().unwrap();
        let min = std::cmp::min(counter.0 as usize, bag.len());
        parallel_commands.command_scope(|mut commands: Commands| {
            for i in 0..min {
                commands.entity(bag[i]).despawn();
            }
        });
    });
}

// T: TODO try to parallelize this method
fn sheeps_eat(mut query_sheeps: Query<(&mut Sheep, &DBRead<Location>)>, 
            mut query_grass_field: Query<&mut DenseSingleValueGrid2D<u16>>) {

    let mut grass_field = query_grass_field.single_mut();

    query_sheeps.iter_mut().for_each(|(mut sheep_data, sheep_loc)| {
        if grass_field.get_value(&sheep_loc.0.0).expect("empty cell of the grass field") == FULL_GROWN {
            grass_field.set_value_location(0, &sheep_loc.0.0);
            sheep_data.energy += GAIN_ENERGY_SHEEP;
        }
    });
}

// Not a problem if a sheep that is killed reproduce, because in the normal iteration a sheep can reproduce and then be killed(so i only inverted)
// the order of this
// A wolf can only die through the reproduction
// TODO verify if it is better to run a query with only Agent, probably is the same
fn reproduce_sheeps(mut query_sheeps: Query<(Entity, &mut Sheep, &DBRead<Location>)>, mut parallel_commands: ParallelCommands) {

    query_sheeps.par_iter_mut().for_each(
  |(entity, mut sheep_data, loc)| {

            let mut rng = rand::thread_rng(); 

            parallel_commands.command_scope(|mut commands| {

                sheep_data.energy -= ENERGY_CONSUME;
            
                if sheep_data.energy > 0. && rng.gen_bool(SHEEP_REPR as f64) {
                    sheep_data.energy /= 2.0;
                    commands.spawn((
                    Sheep {
                        id: 0,
                        energy: GAIN_ENERGY_SHEEP,
                    }, 
        
                    DoubleBuffered::new(Location(loc.0.0)),
                    DoubleBuffered::new(LastLocation(None)),
        
                    Agent,)
                    );
                }
                if sheep_data.energy <= 0. {
                    commands.entity(entity).despawn();
                } 
            });
        }
    );
}   

// T: the best i can make for now
fn reproduce_wolves(mut query_wolfs: Query<(Entity, &mut Wolf, &DBRead<Location>)>, mut parallel_commands: ParallelCommands) {

    query_wolfs.par_iter_mut().for_each(
        |(entity, mut wolf_data, loc)| {
      
                  let mut rng = rand::thread_rng(); 
                  
                  wolf_data.energy -= ENERGY_CONSUME;

                  parallel_commands.command_scope(|mut commands| {
      
                      if wolf_data.energy > 0. && rng.gen_bool(WOLF_REPR as f64) {
                          wolf_data.energy /= 2.0;
                          commands.spawn((
                          Wolf {
                              id: 0 ,
                              energy: GAIN_ENERGY_WOLF,
                          }, 
              
                          DoubleBuffered::new(Location(loc.0.0)),
                          DoubleBuffered::new(LastLocation(None)),
              
                          Agent,)
                          );
                      }
                      if wolf_data.energy <= 0. {
                          commands.entity(entity).despawn();
                      }
                      
                  });
              }
          );
}

// T: Probably the better version possible
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

// Run at the end
fn update_sheeps_field(query_sheeps: Query<(Entity, &Sheep, &DBWrite<Location>)>,
                        mut query_sheeps_field: Query<(&mut DenseBagGrid2D<Entity, SheepField>)>) {

    let mut sheeps_field = query_sheeps_field.single_mut();
    sheeps_field.clear();

    let process_sheep = |(entity, sheep, loc): (Entity, &Sheep, &DBWrite<Location>)| {
        if sheep.energy > 0. {
            sheeps_field.push_object_location(entity, &loc.0.0);
        }
    };

    query_sheeps.iter().for_each(process_sheep);
}

fn grass_grow(mut query_grass_field: Query<(&mut DenseSingleValueGrid2D<u16>)>) {
    // Parallel update of the grass
    // At least this shit(but uses rayon)
    // TODO Test if is good or not
    // TODO insert here some spans
    let mut grass_field = &mut query_grass_field.single_mut();

    grass_field.values.par_iter_mut().for_each(|grass_value| {
        let growth = *grass_value;
        match (*grass_value) {
            Some(grass) => {
                *grass_value = Some(grass + 1);
            },
            None => {

            }
        }
    });
}










fn init_world(mut commands: Commands) {
    println!("init_world!");

    let mut rng = rand::thread_rng();

    // T: generate the grass (START)
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

    commands.spawn((grass_field));
    // T: generate the grass (END)


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

    let sheeps_field = DenseBagGrid2D::<Entity, SheepField>::new(DIM_X as i32, DIM_Y as i32);
    commands.spawn((sheeps_field));
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

    commands.spawn((AtomicGrid2D::<CountWolfs>::new((0u32, 0u32), DIM_X as i32, DIM_Y as i32)));

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