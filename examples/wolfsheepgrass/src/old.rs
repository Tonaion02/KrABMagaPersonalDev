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

        sheeps_field.push_object_location(entity_commands.id(), &loc);
    }

    println!("size sheeps field: {}", sheeps_field.bags.len());
    let mut count = 0usize;
    for i in &mut sheeps_field.bags {
        count = count + i.len();
    }
    println!("size sheeps field total: {}", count);

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

        wolfs_field.push_object_location(entity_command.id(), &loc);
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


// T: version that doesn't use multithreading
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

            let mut result = query_sheeps.get_mut(*sheep);
            let mut sheep_data;

            match result {
                Ok(data) => {
                    sheep_data = data;
                }
                Err(_) => {
                    continue;
                }
            }

            if sheep_data.energy > 0. {
                
                //println!("wolf eating {}\n", sheep.index());

                // T: TODO check if it is useless, probably not
                sheep_data.energy = 0.;
                removed = true;
                wolf.energy += GAIN_ENERGY_WOLF;

                commands.entity(*sheep).despawn();

                // T: exit when we found an alive sheep, each wolf
                // T: eat only a prey(taken from the old simulation)
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

fn reproduce_wolves(mut query_wolfs: Query<(Entity, &mut Wolf, &DBRead<Location>)>, mut parallel_commands: ParallelCommands) {
    
    // T: TEMP for debug purpose
    let mutex_counter = Arc::new(Mutex::new(0u32)); 

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
                          //println!("killing wolf");
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

    let process_sheep = |(entity, sheep, loc): (Entity, &Sheep, &DBWrite<Location>)| {
        if sheep.energy > 0. {
            sheeps_field.push_object_location(entity, &loc.0.0);
        }
    };

    query_sheeps.iter().for_each(process_sheep);
}

// T: In this case i am using DBWrite to run this system in parallel before
// T: the syncing of the double buffering about locations
fn update_wolfs_field(query_wolfs: Query<(Entity, &Wolf, &DBWrite<Location>)>,
                    mut query_wolfs_field: Query<(&mut DenseBagGrid2D<Entity, WolfField>)>) {

    let mut wolfs_field = query_wolfs_field.single_mut();
    wolfs_field.clear();

    let process_wolf = 
    |(entity, wolf, loc): (Entity, &Wolf, &DBWrite<Location>)| {
        if wolf.energy > 0. {
            wolfs_field.push_object_location(entity, &loc.0.0);
        }
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
