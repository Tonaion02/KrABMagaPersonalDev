<center> <h1> MY PERSONAL KRABMAGA DEVELOPMENT REPOSITORY </h1> </center>

The following is my personal repository for development of [KrABMaga](https://krabmaga.github.io/) during my apprenticeship.

My work is focused above trying to squeeze performance of the framework using the [ECS](https://en.wikipedia.org/wiki/Entity_component_system#:~:text=Entity%E2%80%93component%E2%80%93system) of [Bevy](https://bevyengine.org/). 

# EXPLORE MY EXPERIMENTS

Each experiment is contained in a different directory in [examples](https://github.com/Tonaion02/KrABMagaTirocinio_provisory/tree/fix_simulations/examples).

At the moment i have tried to improve performance of the framework working to these simulations:
- Flockers
- WolfSheepGrass

# EXECUTE EXPERIMENTS

To try my experiments you must execute the following command:
```
cargo run --release --features "krabmaga/multithreaded krabmaga/trace_tracy" -- parameters 
```

In this command the list of **features** is specified between quotation marks.
The feature **multithreaded** indicates to use a number of thread equals to the number of threads specified from the command-line argument to execute the simulation.
The feature **trace_tracy** is useful for debugging. It produces the information for the profiler.
<br>

In this command the list of parameters is specified after "--".

The parameters for simulation that you can pass like command-line arguments are(in the order): NUM_THREAD NUM_AGENTS DIM_X STEPS.

Where:
- NUM_THREADS are the number of threads used by the simulation.
- NUM_AGENTS are the number of agents of the simulation.
- DIM_X is the horizzontal dimension of the enviroment.(is used also like vertical dimension of enviroment)
- STEPS is the number of steps to execute for the simulation.

# WOLFSHEEPGRASS EXPERIMENTS

## EXPERIMENT 3

### WHAT I TRIED
For this experiment i tried to parallelize a specific set of operations of this simulation.
At certain point of the step the wolves eats sheeps.
Then i tried to parallelize the updating of the grass.

### TRIVIAL IMPLEMENTATION OF WOLVES EAT SHEEPS
The most trivial implementation is:
- query for sheeps(mutable)
- query for wolves(mutable) and wolf's location
- retrieve the field of the sheeps(mutable)
- for each wolf check in the field if there is a sheep in the wolf's location, if it is true:
    - gain energy to wolf
    - remove the sheep from the field

In this case we need to modify the field to stop other wolves from eating death sheeps. So this operation is really difficult to implement in parallel for the need to notify to other wolves that a certain sheep is not valid to eat.

### NEW IMPLEMENTATION OF WOLVES EAT SHEEPS
I understood that the unique things i care is how many sheeps die and how many wolves gain energy for each location(obviously the same number).

What i tried is to compute in a grid the numbers of wolves for each location with an atomic counter for each location(cell of the grid).

After that i runned in parallel a system on the sheeps' field and the grid, and i used these to despawn the correct amount of sheeps from the grid.

Then i used the number of wolves for each cell, a classic field for the sheeps and a query on the wolves and their location to gain energy for each wolf.
In parallel, for each wolf i compute the minimum beetween the number of sheeps in that location and the number of wolves, decrease the minimum and assign it like new counter in the grid, then i gain energy to wolf if counter is greater than zero.

### NEW IMPLEMENTATION OF GROWING GRASS
I simply used a parallel iterator from [rayon](https://docs.rs/rayon/latest/rayon/) to parallelize the process of updating of the grass.
I need to test if this is convenient in this case, we are talking about a simply vector updating that can easily be performed in easily manners from the compiler.

## EXPERIMENT 4

### WHAT I TRIED
For this experiment i tried another way to modelize the interaction beetween sheeps and wolfs. 
This is not really necessary for this simulation. In this simulation is not necessary that a wolf specifically eat a specifically sheep. In other simulation where is important that, we must gave the possibility to do very rapidly this.

### TRIVIAL IMPLEMENTATION
In the case of this simulation, the implementation that i have tried in the experiment_3 may be enough.
In theory we only know that each wolf try to eat a sheep that is in the same location. So we can directly count the number of wolfs for location, the number of sheeps for location, and the number of sheep that has be eaten/wolves that gain energy is equal to the minimum of the two counter for each location.

This method cannot be applied to more complex simulations. In many simulations we need for example that an agents choose the other agents with some euristic or logic.
In this experiment i'm trying to find a solution that can work also in these other more complex simulations.

### NEW IMPLEMENTATION
I mantained the constraint that every agents interact only with the agents in the same location.
The approach is to loop in parallel among the two DenseGrid(One for the sheep and another for the wolves). So for each bag of wolves and sheep that reside in the same location i need for each one wolves to select an alive sheep(if there is) to kill. Each wolves that kill a sheep, gain energy from that.
I haven't used this technique in the previous experiments because there is a problem. To apply this tecnique i need at least to modify the data of the wolves(the energy of each wolves that has eatean a sheep).

I canno't use a mutable query object from Bevy to do that. Because mutable object cannot be used in parallel context like shared object.

I am sure that in this parallel iteration i'm going to modify different entities each time. I am sure of that because the entities are unique in the DenseGrids.
So i only need a method to indicate to the Rust compiler that i'm sure that my code doesn't access the same entities from different threads.

To do that i can use some lock. With lock i give the assurance to the compiler the assurance that multiple threads cannot access to the same memory location at the same time.

If i put a lock directly on mutable query object, the code became really slow. This because there is a race condition on an object(the query) that must be used every time. This lead to the fact that every threads lock the same object. The code can be slower than a sequential version of the code(I haven't tried this approach cause i don't need to test to know that is really really slow).

The idea is to put lock directly on the data of the agents that must be accessed in parallel context. There isn't really race condition, only one thread at time only one time acquire the lock and release the lock. This mutex is only necessary to Rust to be sure that multiple threads cannot access to the same memory location at the same time, even if we are sure of that for a constraint of the problem, we must give to rust this assurance.

## EXPERIMENT 5

### WHAT I TRIED
With the previous experiment i have obtained a speed-up and a manner to handle complex interaction within multiple agents.

Before this experiment i didn't tried to parallelize many operations.
Some sequential operations are:
- Update of dense grids
- Adding/Removing entities from ECS storing
- Sheep eating grass

In this experiment i tried to parallelize the update of dense grids. 

### PREVIOUS TRIVIAL IMPLEMENTATION
In the previous and trivial implementation of this operation, i rebuild from zero the entire grid with a sequential approach.

So we clear all the bags of the dense grid. The clear operation doesn't allocate memory, we just reset some index.
Then each agents is inserted in a bag of the grid. 

Rebuild these dense grids cannot be easily parallelized because we need for each agents of the pool to insert(and so modify) the grid. This is a race condition among the grid for each agent.

### NEW IMPLEMENTATION
We can't easily solve the problem putting a lock on the grid. Because each agent make a race condition among the grid.

If we see the problem from another prospective, we can say that each agent make a race condition among a bag of the grid. We don't modify the grid or the vector where the bags reside. For each agent we only modify one bag.

The idea is to put a lock on each bag. With this technique we can lock only the bag where the agent must be inserted. With this technique we easily reduce the number of threads that wait other threads to release the locks.

## EXPERIMENT 6

### WHAT I TRIED
With the previous experiment i have tried to obtain a speed-up parallelizing the operation of re-creating a field(DenseGrid).

I haven't noticed a notable speedup after this operation.
I have thinked that is because for race conditions about the bag for each cell, but during profiling i have noticed something strange:
<img title="profiling_image_exp_6" alt="profiling_image_exp_6" src="experiments_doc/wolfsheepgrass/profiling_image_exp_6.PNG">
From this trace we can see that all the threads is used only during the last part of recreating a field.
This suggest that there is another operation that takes the major part of the time.
How we can see the only other part of this method is the clearing of update field.
```rust
fn update_wolves_field(
    query_wolfs: Query<(Entity, &Wolf, &DBWrite<Location>)>, 
    mut query_wolfs_field: Query<(&mut ParDenseBagGrid2D_exp_6<Entity, WolfField>)>,
) {

    let mut wolfs_field = query_wolfs_field.single_mut();
    
    // The last part that remains is the clear
    // In fact this operation is realized in sequential on each cell of the grid
    wolfs_field.clear();

    // This part is executed in parallel in the last part of the profiling (START)
    let process_wolf = |(entity, wolf, loc) : (Entity, &Wolf, &DBWrite<Location>)| {

        let mut wolf_bag = wolfs_field.get_write_bag(&loc.0.0);
        wolf_bag.push(entity);
    };
    // The last part that remains is the clear (END)

    query_wolfs.par_iter().for_each(process_wolf);
    // This part is executed in parallel in the last part of the profiling (END)    
}
```

### NEW IMPLEMENTATION
Simple we will try to parallelize this operation.

## EXPERIMENT 7

### WHAT IS THE PROBLEM NOW
After we have solved/mitigated the problem of updating fields during the simulation migrating it to parallel, we have left one great operation like a sequential inefficient block: the deferred applying of commands.

We have used [ParallelCommands](https://docs.rs/bevy/latest/bevy/prelude/struct.ParallelCommands.html) to delay the insertion/deletion from pool of entities to not create race condition about the entire structure during the process of reproduction/killing of elements that is realized in parallel.
All the commands that we create is then applyed in sequential at the end of step/frame.
<img title="profiling_image_exp_7" alt="profiling_image_exp_7" src="experiments_doc/wolfsheepgrass/profiling_image_exp_7.PNG">
This create some problem, this part takes more time than the entire simulation.

### NEW IMPLEMENTATION
The idea is to stop using ParallelCommands to save the agents to spawn.
We can easily optimize the spawning of new agents with [spawn_batch](https://docs.rs/bevy/latest/bevy/ecs/prelude/struct.Commands.html#method.spawn_batch). We can pass an iterator to spawn_batch for the data we want to use to initialize the new agents(entities in general).

So the idea is to accumulate all the data for the new agents in a single buffer to initialize then the new agents.
But we must share this buffer in a parallel context, so we tried to use [Parallel](https://docs.rs/bevy/latest/bevy/utils/struct.Parallel.html) from bevy_utils.

<br>

# FLOCKERS EXPERIMENTS

# FIXED VISUALIZATION

Flockers is the first simulation that is re-developed with the ECS(not by me, but by [Carbon](https://github.com/carbonhell)). The visualization part is break because even is written in Bevy rely on the Object-Based architecture of original KrABMaga. The idea is to re-implement the visualization to explore the possible solution to deliver to people a solution to implement rapidly a visualization for their simulations. 