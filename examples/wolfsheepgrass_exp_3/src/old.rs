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
    
    app.add_systems(Update, count_agents.in_set(BeforeStep));
    app.add_systems(Update, count_sheeps.in_set(BeforeStep));
    app.add_systems(Update, count_wolfs.in_set(BeforeStep));
    app.add_systems(Update, print_step.in_set(BeforeStep).before(count_wolfs).before(count_sheeps).before(count_agents));

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



    // T: TEST for rng
    let mut rng = RNG::new(simulation_descriptor.rand_seed, simulation_descriptor.current_step);
    let gen_float = rng.gen();
    let gen_bool = rng.gen_bool(0.2);
    let gen_float_in_range = rng.gen_range(0f32 .. 1f32);

    println!("generated float: {}", gen_float);
    println!("generated bool: {}", gen_bool);
    println!("generated float in range: {}", gen_float_in_range);
    // T: TEST for rng



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
        if grass_field.get_value(&sheep_loc.0.0).expect("empty cell(not possible!)") == FULL_GROWN {
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
    // T: TEST for now is not parallel for debugging

    #[cfg(any(feature = "debug_support"))]
    let mut counter_for_coin_flip = 0u32;
    #[cfg(any(feature = "debug_support"))]
    let mut counter_dead_for_out_energy_sheeps = 0u32;

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
                counter_for_coin_flip += 1;
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
                { counter_dead_for_out_energy_sheeps += 1; }
            }

        });

    });

    std::mem::drop(span);

    #[cfg(any(feature = "debug_support"))]
    println!("results coin_flip: {}", counter_for_coin_flip);
    #[cfg(any(feature = "debug_support"))]
    println!("dead sheeps for out of energy {}", counter_dead_for_out_energy_sheeps);
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

    #[cfg(any(feature = "debug_support"))]
    let mut counter_non_zero_counters = 0u32;
    #[cfg(any(feature = "debug_support"))]
    for counter in & grid.values {
        if *counter.lock().unwrap() > 0 {
            counter_non_zero_counters += 1;
        }
    }
    #[cfg(any(feature = "debug_support"))]
    println!("counter_non_zero_counters(before): {}", counter_non_zero_counters);

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

    #[cfg(any(feature = "debug_support"))]
    let mut counter_non_zero_counters = 0u32;
    #[cfg(any(feature = "debug_support"))]
    for counter in & grid.values {
        if *counter.lock().unwrap() > 0 {
            counter_non_zero_counters += 1;
        }
    }
    #[cfg(any(feature = "debug_support"))]
    println!("counter_non_zero_counters(after): {}", counter_non_zero_counters);

    std::mem::drop(span_internal);

    std::mem::drop(span);
    // T: Wolves eat (END)



    // T: Reproduce wolves (START)

    #[cfg(any(feature = "debug_support"))]
    let mut count_dead_wolfs = 0u32;

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

                      if wolf_data.energy > 0. &&  gen_bool {

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
                          { count_dead_wolfs += 1; }
                      }
                      
                  });
              }
          );
    
    #[cfg(any(feature = "debug_support"))]
    println!("dead wolfs: {}", count_dead_wolfs);
        

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

    // TEMP for debug purpose
    #[cfg(any(feature = "debug_support"))]
    let mut verify_counter = 0;
    #[cfg(any(feature = "debug_support"))]
    for element in & grid.values {
        verify_counter += *element.lock().unwrap();
    }
    #[cfg(any(feature = "debug_support"))]
    println!("total wolf in the grid: {}", verify_counter);
    // TEMP for debug purpose
}

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

    // TEMP for debug purpose
    #[cfg(any(feature = "debug_support"))]
    let mut non_empty_bags_counter = 0u32;

    #[cfg(any(feature = "debug_support"))]
    for bag in & sheeps_field.bags {
        if ! bag.is_empty() {
            non_empty_bags_counter += 1;
        }
    }

    #[cfg(any(feature = "debug_support"))]
    println!("non empty bags: {}", non_empty_bags_counter);
    // TEMP for debug purpose
}

fn grass_grow(mut query_grass_field: Query<(&mut DenseSingleValueGrid2D<u16>)>) {
    // Parallel update of the grass
    // At least this shit(but uses rayon)
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
            //#[cfg(any(feature="fixed_random"))]
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
        let initial_energy = rng.gen_range(0.1 ..(2. * GAIN_ENERGY_SHEEP));
        //println!("{}", initial_energy);

        let entity_commands = commands.spawn((
            Sheep {
                id: id_to_assign,
                energy: initial_energy,
            }, 

            DoubleBuffered::new(Location(loc)),
            DoubleBuffered::new(LastLocation(None)),

            Agent { id: sheep_id + NUM_INITIAL_WOLFS },
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
        let initial_energy = rng.gen_range(0.1 ..(2. * GAIN_ENERGY_WOLF));

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
