//T: temp
#![allow(warnings)]



use std::time::Instant;

use std::env;
use krabmaga::engine::resources::simulation_descriptor;
use lazy_static::lazy_static;

use std::path::Path;
use std::fs::File;
use std::io::prelude::*;

use core::f32::consts::PI;

// T: importing rayon (START)
extern crate rayon;
use crate::rayon::iter::IntoParallelRefIterator;
use crate::rayon::iter::IntoParallelRefMutIterator;
use crate::rayon::iter::ParallelIterator;
use crate::rayon::iter::IndexedParallelIterator;
// T: importing rayon (END)

use krabmaga::engine::Entity;
use krabmaga::engine::Query;
use krabmaga::engine::Res;
use krabmaga::engine::agent::AgentFactory;
use krabmaga::engine::components::double_buffer::DBRead;
use krabmaga::engine::components::double_buffer::DBWrite;
use krabmaga::engine::components::position::Real2DTranslation;
use krabmaga::engine::fields::field_2d::Field2D;
// T: TODO move this methods to the right place (START)
use krabmaga::engine::fields::field_2d::toroidal_distance; 
use krabmaga::engine::fields::field_2d::toroidal_transform;
// T: TODO move this methods to the right place (END)
use krabmaga::engine::location::Real2D;
use krabmaga::engine::location::Int2D;
use krabmaga::engine::resources::simulation_descriptor::SimulationDescriptorT;
use krabmaga::engine::rng::RNG;
use krabmaga::engine::simulation::Simulation;
use krabmaga::engine::Commands;
use krabmaga::engine::components::double_buffer::DoubleBuffered;

use krabmaga::engine::bevy_prelude::*;

use krabmaga::engine::Update;
use krabmaga::engine::simulation::SimulationSet::Step;
use krabmaga::engine::simulation::SimulationSet::AfterStep;
use krabmaga::engine::simulation::SimulationSet::BeforeStep;

use krabmaga::engine::bevy_prelude::IntoSystemSetConfigs;

use crate::model::bird::Bird; 
use crate::model::bird::LastReal2D;

use krabmaga::engine::fields::parallel_dense_object_grid_2d_flockers_exp_1::ParDenseBagGrid2D_flockers_exp_1;

mod model;

// T: For visualization
#[cfg(any(feature = "visualization"))]
mod visualization;

#[cfg(any(feature = "visualization"))]
use krabmaga::visualization::visualization::Visualization;

#[cfg(any(feature = "visualization"))]
use krabmaga::engine::agent::Agent;
#[cfg(any(feature = "visualization"))]
use krabmaga::engine::Commands;
#[cfg(any(feature = "visualization"))]
use krabmaga::engine::With;

#[cfg(any(feature = "visualization"))]
use krabmaga::visualization::AssetServer;
#[cfg(any(feature = "visualization"))]
use krabmaga::visualization::Transform;
#[cfg(any(feature = "visualization"))]
use krabmaga::visualization::SpriteBundle;
#[cfg(any(feature = "visualization"))]
use krabmaga::visualization::Vec3;
#[cfg(any(feature = "visualization"))]
use krabmaga::visualization::Quat;
#[cfg(any(feature = "visualization"))]
use krabmaga::visualization::Color;
// T: For visualization




// T: Constants (START)
pub static COHESION: f32 = 0.8;
pub static AVOIDANCE: f32 = 1.0;
pub static RANDOMNESS: f32 = 1.1;
pub static CONSISTENCY: f32 = 0.7;
pub static MOMENTUM: f32 = 1.0;
pub static JUMP: f32 = 0.7;
pub static DISCRETIZATION: f32 = 10.0 / 1.5;
pub static TOROIDAL: bool = true;
//pub static STEPS: u32 = 100;
pub static SEED: u64 = 1337;

pub const SIMULATION_TITLE: &'static str = "Flockers_exp_1";

// T: New Constants (START)
// MODIFIED: now we retrieve this parameters from command line
// but that acts like "static constants", little trick from here:
// https://stackoverflow.com/questions/37405835/populating-a-static-const-with-an-environment-variable-at-runtime-in-rust
lazy_static! {
    static ref NUM_THREADS: usize = 
    match (std::env::args().collect::<Vec<String>>().get(1)) {
        Some(value) => { value.clone().parse::<usize>().unwrap() }
        None => { 0usize }
    };

    static ref NUM_AGENTS: u32 = 
    match (std::env::args().collect::<Vec<String>>().get(2)) {
        Some(value) => { value.clone().parse::<u32>().unwrap() }
        None => { 0u32 }
    };

    static ref DIM_X: f32 = 
    match (std::env::args().collect::<Vec<String>>().get(3)) {
        Some(value) => { value.clone().parse::<f32>().unwrap() }
        None => { 0. }
    };

    static ref STEPS: u32 =
    match (std::env::args().collect::<Vec<String>>().get(4)) {
        Some(value) => { value.clone().parse::<u32>().unwrap() }
        None => { 0u32 }
    };

    static ref DIM_Y: f32 = *DIM_X;
}
// T: New Constants (END)
// T: Constants (START)





// T: Define markers for fields (START)
pub struct FlockerGrid;
// T: Define markers for fields (END)

// Main used when only the simulation should run, without any visualization.
#[cfg(not(any(feature = "visualization", feature = "visualization_wasm")))]
fn main() {
    let mut simulation = build_simulation(Simulation::build().with_steps(*STEPS));
    let now = Instant::now();
    simulation.run();
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}, steps per second: {}", elapsed, *STEPS as f64 / elapsed.as_secs_f64());
    
    save_elapsed_time(elapsed);    
}
// Main used when only the simulation should run, without any visualization.



fn build_simulation(mut simulation: Simulation) -> Simulation {
    // T: commented from me
    // let field: Field2D<Entity> = Field2D::new(*DIM_X, *DIM_Y, DISCRETIZATION, TOROIDAL);

    // T: Setting rayon's enviroment variable (START)
    // T: TODO move this in the correct place

    rayon::ThreadPoolBuilder::new().
    num_threads(*NUM_THREADS).

    // start_handler(|real_thread_id| {
    //     thread_id.with(|cell| { cell.set(real_thread_id); });
    // }).
    build_global().
    unwrap();
    // T: Setting rayon's enviroment variable (END)


    let mut simulation = simulation
        .with_title(String::from(SIMULATION_TITLE))
        .register_double_buffer::<Real2DTranslation>()
        .register_double_buffer::<LastReal2D>()
        // T: commented because outdated
        // .register_step_handler(step_system)
        .with_num_threads(*NUM_THREADS)
        .with_simulation_dim(Real2D {x: *DIM_X, y: *DIM_Y});
        // .with_rng(SEED) // We cannot use this during parallel iteration due to mutable access being required for RNG.
        
        // T: commented for error
        //.with_engine_configuration(EngineConfiguration::new(Real2D { x: *DIM_X, y: *DIM_Y }, SEED)); // TODO abstract
    
    // TODO figure out how configs should work. Either split engine config and simulation config, requiring the latter to be registered, or...?
    //init_world(&mut simulation, field);

    simulation = simulation.register_init_world(init_world);

    let app = &mut simulation.app;

    app.add_systems(Update, step_system.in_set(Step));
    app.add_systems(Update, update_agents_grid.in_set(BeforeStep));

    simulation
}



// TODO couple DBRead and DBWrite queries in a single systemparam
// TODO assume step systems will always query all the components added to an entity and make a systemparam grouping all of them automatically? Splitting up will hardly matter since inner parallelism with par queries will always be better
// TODO compare with 2024 flockers step code
fn step_system(
    mut query: Query<(Entity, &Bird, &DBRead<Real2DTranslation>, &DBRead<LastReal2D>, &mut DBWrite<Real2DTranslation>, &mut DBWrite<LastReal2D>)>,
    neighbour_query: Query<(&DBRead<Real2DTranslation>, &DBRead<LastReal2D>)>,

    query_grid: Query<( &ParDenseBagGrid2D_flockers_exp_1<Entity, FlockerGrid>)>,

    // T: Commented because outdated
    // field_query: Query<&Field2D<Entity>>,
    
    config: Res<SimulationDescriptorT>
) {

    // T: Commented because outdated
    // let field = field_query.single();
    let grid = query_grid.single();
    
    
    //println!("Step #{}", config.current_step);
    let now = Instant::now();
    query.par_iter_mut().for_each(|(entity, bird, cur_pos, last_pos, mut w_cur_pos, mut w_last_pos)| {
        let cur_pos = cur_pos.0.0;
        let last_pos = last_pos.0.0;
        
        // T: TODO i can remove this shitty abit to make a retain among the vector
        // T: probably i can use a particular version of get_neightbours that can
        // T: check what entity you can put in.
        // T: TODO verify if we can evitate to re-allocate each time buffer to store
        // T: neighbours. Probably we can use: https://doc.rust-lang.org/std/macro.thread_local.html 
        // T: commented because outdated
        // let mut neighbours = field.get_neighbors_within_relax_distance(cur_pos, 10.);
        let mut neighbours = grid.get_neighbors_within_relax_distance(cur_pos, 10.);
        neighbours.retain(|x| *x != entity);
        
        let (mut x_avoidance, mut y_avoidance) = (0., 0.);
        let (mut x_cohesion, mut y_cohesion) = (0., 0.);
        let (mut x_consistency, mut y_consistency) = (0., 0.);
        let (mut x_randomness, mut y_randomness) = (0., 0.);
        let (x_momentum, y_momentum) = (last_pos.x, last_pos.y);
        // Previously we had a check for neighbours being empty, but the check was actually pointless since the vec always contained at least {bird}.
        let mut count = 0;
        for (elem_loc, last_elem_loc) in neighbour_query.iter_many(neighbours) {
            let elem_loc = elem_loc.0.0;
            let last_elem_loc = last_elem_loc.0.0;
            
            let dx = toroidal_distance(cur_pos.x, elem_loc.x, *DIM_X);
            let dy = toroidal_distance(cur_pos.y, elem_loc.y, *DIM_Y);
            count += 1;
            
            //avoidance calculation
            let square = dx * dx + dy * dy;
            x_avoidance += dx / (square * square + 1.0);
            y_avoidance += dy / (square * square + 1.0);
            
            //cohesion calculation
            x_cohesion += dx;
            y_cohesion += dy;
            
            //consistency calculation
            x_consistency += last_elem_loc.x;
            y_consistency += last_elem_loc.y;
        }
        //println!("Elapsed 2 agent {}: {:?}", bird.id, now.elapsed());
        if count > 0 {
            x_avoidance /= count as f32;
            y_avoidance /= count as f32;
            x_cohesion /= count as f32;
            y_cohesion /= count as f32;
            x_consistency /= count as f32;
            y_consistency /= count as f32;
            
            x_consistency /= count as f32;
            y_consistency /= count as f32; // Old code did this division twice
        }
        
        x_avoidance *= 400.;
        y_avoidance *= 400.;
        x_cohesion = -x_cohesion / 10.;
        y_cohesion = -y_cohesion / 10.;
        // We cannot cache the RNG during parallel iteration since generating a number requires mutating the RNG itself.
        // Finding a way to assign one rng per thread generated by par_iter is probably overkill considering the little gain it provides.
        let mut rng = RNG::new(config.rand_seed, bird.id as u64 + config.current_step as u64);
        rng.set_stream(bird.id as u64 + config.current_step as u64);
        let r1 = rng.gen() * 2. - 1.;
        let r2 = rng.gen() * 2. - 1.;
        let square = (r1 * r1 + r2 * r2).sqrt();
        x_randomness = 0.05 * r1 / square;
        y_randomness = 0.05 * r2 / square;
        //println!("Elapsed 3 agent {}: {:?}", bird.id, now.elapsed());
        let mut dx = COHESION * x_cohesion
        + AVOIDANCE * x_avoidance
        + CONSISTENCY * x_consistency
        + RANDOMNESS * x_randomness
        + MOMENTUM * x_momentum;
        let mut dy = COHESION * y_cohesion
        + AVOIDANCE * y_avoidance
        + CONSISTENCY * y_consistency
        + RANDOMNESS * y_randomness
        + MOMENTUM * y_momentum;
        
        let dis = (dx * dx + dy * dy).sqrt();
        if dis > 0.0 {
            dx = dx / dis * JUMP;
            dy = dy / dis * JUMP;
        }   

            let loc_x = toroidal_transform(cur_pos.x + dx, *DIM_X);
            let loc_y = toroidal_transform(cur_pos.y + dy, *DIM_Y);
            /* if config.current_step == 200 {
                println!("Bird {} - Step {}: - cohesion {:?}, avoidance {:?}, consistency {:?}, randomness {:?}, mom {:?}, loc {:?}",
                bird.id, config.current_step, (x_cohesion, y_cohesion), (x_avoidance,y_avoidance), (x_consistency,y_consistency), (x_randomness, y_randomness),
                (x_momentum, y_momentum), (loc_x, loc_y));
                } */

               
               // TODO this is ugly, but if we unify read and write buffers we'll end up querying both all the time even when it's not needed
               // TODO perhaps give the user a way to query only read or both read and write, and proxy methods accordingly
               w_last_pos.0 = LastReal2D::new(Real2D { x: dx, y: dy });
               w_cur_pos.0 = Real2DTranslation(Real2D { x: loc_x, y: loc_y });
               //println!("Elapsed 4 agent {}: {:?}", bird.id, now.elapsed());
            });
    //println!("Elapsed: {:?}", now.elapsed());
}

// T: Run before step (START)
fn update_agents_grid(
    query_agents: Query<(Entity, &Bird, &DBWrite<Real2DTranslation>)>,
    mut query_grid: Query<(&mut ParDenseBagGrid2D_flockers_exp_1<Entity, FlockerGrid>)>,
) {

    let mut grid = query_grid.single_mut();
    grid.clear();

    query_agents.par_iter().for_each(|(entity, bird, loc)| {

        // T: TODO add the missing discretization(WARNING: probably it's the cause of the random crashes)
        let loc = grid.discretize(&loc.0.0);

        let mut bag = grid.get_write_bag(&loc);
        bag.push(entity);
    });

}
// T: Run before step (END)





// TODO: remove this. The user should specify a bundle representing the agent (AgentBundle) with 
// TODO: the component it requires.
// TODO: there needs to be a way to specify initialization logic too though.
// TODO: this bundle prototype must be passed to the simulation so that, along with NUM_AGENTS, the simulation
// TODO: can be programmatically restarted.
fn init_world(

    simulation_descriptor: Res<SimulationDescriptorT>,
    mut commands: Commands,

    // T: Removed from me
    // simulation: &mut Simulation, 
    // field: Field2D<Entity>,
) {
    // T: Commented from me (START)
    // for bird_id in 0..*NUM_AGENTS {
    //     let mut rng = RNG::new(SEED, bird_id as u64);
    //     let r1: f32 = rng.gen();
    //     let r2: f32 = rng.gen();
        
    //     let position = Real2D { x: *DIM_X * r1, y: *DIM_Y * r2 };
    //     let current_pos = Real2DTranslation(position);
        
    //     let mut agent = AgentFactory::new(simulation);
        
    //     agent
    //     .insert_data(Bird { id: bird_id })
    //     .insert_double_buffered(LastReal2D::new(Real2D { x: 0., y: 0. }))
    //     .insert_double_buffered(current_pos);
    // //println!("Bird #{} init: ({}, {}), {} - {}", bird_id, position.x, position.y, r1, r2);
    // }

    // simulation.add_field(field);
    // T: Commented from me (END)

    for bird_id in 0..*NUM_AGENTS {
        let mut rng = RNG::new(SEED, bird_id as u64);
        let r1: f32 = rng.gen();
        let r2: f32 = rng.gen();

        // T: TODO adjust the position
        let position = Real2D { x: rng.gen_range(0. .. *DIM_X), y: rng.gen_range(0. .. *DIM_Y) };
        if (position.x as i32 + (position.y as i32) * *DIM_X as i32) > *DIM_X as i32 * *DIM_Y as i32 {
           println!("ERROR!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!"); 
        }

        let current_pos = Real2DTranslation(position);

        let entity_commands = commands.spawn((
            Bird {id: bird_id},

            DoubleBuffered::new(LastReal2D::new(Real2D {x: 0., y: 0.})),
            DoubleBuffered::new(current_pos),
        ));
    }

    let mut grid = ParDenseBagGrid2D_flockers_exp_1::<Entity, FlockerGrid>::new(*DIM_X as i32, *DIM_Y as i32, DISCRETIZATION, TOROIDAL);
    commands.spawn((grid));
}





#[cfg(any(feature = "visualization", feature = "visualization_wasm"))]
fn graphic_initializer(
    mut commands: Commands,
    query_agents: Query<(Entity, &DBWrite<Real2DTranslation>), (With<Bird>)>,
    asset_server: Res<AssetServer>,
) {
    
    println!("graphic_initalizer is executed!");
    
    for (entity_id, cur_pos) in &query_agents {
        
        let mut transform = Transform::default();
        transform.translation = Vec3::new(cur_pos.0.0.x, cur_pos.0.0.y, 0.);
        transform.scale.x = 0.1;
        transform.scale.y = 0.1;
        
        commands.entity(entity_id).insert(
        SpriteBundle {
            transform: transform,
            texture: asset_server.load("emojis/bird.png"),
            ..Default::default()
        });
    }
}

#[cfg(any(feature = "visualization", feature = "visualization_wasm"))]
fn render_system(
    mut query_agents: Query<(&Bird, &DBWrite<Real2DTranslation>, &DBWrite<LastReal2D>, &mut Transform)>,
) {

    // println!("render_system is running!");
    
    for (bird_id, cur_pos, last_d, mut transform) in &mut query_agents {
        
        transform.translation.x = cur_pos.0.0.x;
        transform.translation.y = cur_pos.0.0.y;

        // T: Compute rotation
        // T: (the computation is taken from the files of the previuos version)
        let mut rotation = if last_d.0.0.x == 0. || last_d.0.0.y == 0. {
            0.
        } else {
            last_d.0.0.y.atan2(last_d.0.0.x)
        };
        rotation = rotation + PI;
        // T: Compute rotation
        
        transform.rotation = Quat::from_rotation_z(rotation);
    }
}



// T: main used only with visualization (START)
#[cfg(any(feature = "visualization", feature = "visualization_wasm"))]
fn main()
{
    let mut simulation = build_simulation(Simulation::build());

    Visualization::default()
    .with_name("flockers_modified")
    .with_window_dimensions(1000., 700.)
    .with_background_color(Color::rgb(0.5, 0.5, 0.5))
    .setup(&mut simulation, graphic_initializer, render_system);

    let now = Instant::now();
    simulation.run();
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}, steps per second: {}", elapsed, *STEPS as f64 / elapsed.as_secs_f64());
       
    save_elapsed_time(elapsed);
}
// T: main used only with visualization (END)





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