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

// T: Constants(START)
pub const ENERGY_CONSUME: f64 = 1.0;

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
    app.add_systems(Update, update_wolfs_field);
    app.add_systems(Update, update_sheeps_field);
    app.add_systems(Update, move_agents.before(update_wolfs_field).before(update_sheeps_field));
    app.add_systems(Update, sheeps_eat.after(update_sheeps_field).after(update_wolfs_field));
    app.add_systems(Update, wolfs_eat.after(update_sheeps_field).after(update_wolfs_field));
    app.add_systems(Update, grass_grow);
    app.add_systems(Update, reproduce_sheeps);
    app.add_systems(Update, reproduce_wolves);
    // T: TEMP
    // T: only for debug purpose
    //app.add_systems(Update, count_agents);

    simulation
}


// T: TEMP
// T: TODO trying to use the fucking AgentFactory that probably is the best
// T: idea.
pub fn insert_double_buffered<T: Component + Copy>(mut entity: EntityWorldMut, value: T) {
    entity.insert(DoubleBuffered::new(value));
}

pub fn count_agents(query_agents: Query<(&Agent)>) {

    let mut count = 0u32;

    query_agents.for_each(|(agent)| {
        count = count + 1;
    });

    println!("{}", count);
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

    commands.spawn((grass_field));
    // T: generate grass (END)

    let mut rng = rand::thread_rng();

    // T: generate sheeps (START)
    println!("generate sheeps");

    let mut sheeps_field = DenseBagGrid2D::<Entity, SheepField>::new(DIM_X as i32, DIM_Y as i32);

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

        sheeps_field.set_object_location(entity_commands.id(), &loc);
    }

    commands.spawn((sheeps_field));
    // T: generate sheeps (END)

    // T: generate wolfs (START)
    println!("genereate wolfs");

    let mut wolfs_field = DenseBagGrid2D::<Entity, WolfField>::new(DIM_X as i32, DIM_Y as i32);

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

        wolfs_field.set_object_location(entity_command.id(), &loc);
    }

    commands.spawn((wolfs_field));
    // T: generate wolfs (END)
}

// T: TODO register modifies in the fields(or add function to recreate fields)
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

fn sheeps_eat(mut query_sheeps: Query<(&mut Sheep, &DBRead<Location>)>, 
              mut query_grass_field: Query<(&mut DenseSingleValueGrid2D<u16>)>) {

    let mut grass_field = query_grass_field.get_single_mut().expect("msg");

    query_sheeps.iter_mut().for_each(|(mut sheep, loc)| {
        
        // T: TODO check if it is necessary to check that value is not written in these iteration or we
        // T: can work on the old iteration values
        // T: I don't know exactly what is the limit on working with the old iteration
        // T: to not create inconsistent situations
        // T: these is particular difficult to parallelize, for what we must use DoubleBuffering?
        if let Some(grass_value) = grass_field.get_value(&loc.0.0) {
            // T: Why >= and not = ??? from the old simulation
            if grass_value >= FULL_GROWN {
                grass_field.set_value_location(0, &loc.0.0);
                sheep.energy += GAIN_ENERGY_SHEEP;
            }
        }
    });
}

fn wolfs_eat(mut query_wolfs: Query<(&mut Wolf, &DBRead<Location>)>, 
            mut query_sheeps: Query<(&mut Sheep)>, 
            mut query_sheeps_field: Query<(&mut DenseBagGrid2D<Entity, SheepField>)>, 
            mut commands: Commands) {

    let mut sheeps_field = query_sheeps_field.get_single_mut().expect("Error retrieving sheeps field");
    
    query_wolfs.iter_mut().for_each(|(mut wolf, wolf_loc) | {

        let mut sheeps_near = sheeps_field.get_ref_mut_bag(&wolf_loc.0.0);
        let mut index = 0u32;
        let mut removed = false;
        for sheep  in sheeps_near {

            let mut sheep_data = query_sheeps.get_mut(*sheep).expect("msg");
            if sheep_data.energy > 0. {
                // T: TEMP
                //To don't give oscillation of population for now
                // T: TODO check if it is useless, probably not
                sheep_data.energy = 0.;
                removed = true;
                wolf.energy += GAIN_ENERGY_WOLF;

                // T: remove with parallel commands the sheeps
                commands.entity(*sheep).despawn();


                // T: exit when we found an alive sheep
                break;
            }

            index += 1u32;
        }

        sheeps_near = sheeps_field.get_ref_mut_bag(&wolf_loc.0.0);
        if removed {
            sheeps_near.swap_remove(index as usize);
        }
    });
}

fn reproduce_sheeps(mut query_sheeps: Query<(Entity, &mut Sheep, &DBRead<Location>)>, mut parallel_commands: ParallelCommands) {


    query_sheeps.par_iter_mut().for_each(
  |(entity, mut sheep_data, loc)| {

            let mut rng = rand::thread_rng(); 


            parallel_commands.command_scope(|mut commands| {

                sheep_data.energy -= GAIN_ENERGY_SHEEP;
            
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
                if sheep_data.energy == 0. {
                    commands.entity(entity).despawn();
                }
                
            });
            
        }
    );

}   

fn reproduce_wolves(mut query_wolfs: Query<(Entity, &mut Wolf, &DBRead<Location>)>, mut parallel_commands: ParallelCommands) {
    
    query_wolfs.par_iter_mut().for_each(
        |(entity, mut wolf_data, loc)| {
      
                  let mut rng = rand::thread_rng(); 
      
      
                  parallel_commands.command_scope(|mut commands| {
      
                    wolf_data.energy -= GAIN_ENERGY_WOLF;
                  
                      if wolf_data.energy > 0. && rng.gen_bool(WOLF_REPR as f64) {
                          wolf_data.energy /= 2.0;
                          commands.spawn((
                          Sheep {
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

// T: In this case i am using DBWrite to run this system in parallel before
// T: the syncing of the double buffering about locations
fn update_sheeps_field(query_sheeps: Query<(Entity, &Sheep, &DBWrite<Location>)>,
                        mut query_sheeps_field: Query<(&mut DenseBagGrid2D<Entity, SheepField>)>) {

    let mut sheeps_field = query_sheeps_field.single_mut();
    sheeps_field.clear();

    let process_sheep = |(entity, sheep_id, loc): (Entity, &Sheep, &DBWrite<Location>)| {
        sheeps_field.set_object(entity, &loc.0.0);
    };

    query_sheeps.iter().for_each(process_sheep);
}

// T: In this case i am using DBWrite to run this system in parallel before
// T: the syncing of the double buffering about locations
fn update_wolfs_field(query_wolfs: Query<(Entity, &Wolf, &DBRead<Location>)>,
                    mut query_wolfs_field: Query<(&mut DenseBagGrid2D<Entity, WolfField>)>) {

    let mut wolfs_field = query_wolfs_field.single_mut();
    wolfs_field.clear();

    let process_wolf = 
    |(entity, wolf_id, loc): (Entity, &Wolf, &DBRead<Location>)| {
        wolfs_field.set_object(entity, &loc.0.0);
    };

    query_wolfs.iter().for_each(process_wolf);
}

// T: TODO check if we need double buffering for grass field
fn grass_grow(mut query_grass_field: Query<(&mut DenseSingleValueGrid2D<u16>)>) {

    let mut grass_field = &mut query_grass_field.single_mut();

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