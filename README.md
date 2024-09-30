<center> <h1> MY PERSONAL KRABMAGA DEVELOPMENT REPOSITORY </h1> </center>

The following is my personal repository for development of [KrABMaga](https://krabmaga.github.io/) during my apprenticeship.

My work is focused above trying to squeeze performance of the framework using the [ECS](https://en.wikipedia.org/wiki/Entity_component_system#:~:text=Entity%E2%80%93component%E2%80%93system) of [Bevy](https://bevyengine.org/). 

## EXPLORE MY EXPERIMENTS

Each experiment is contained in a different directory in [examples](https://github.com/Tonaion02/KrABMagaTirocinio_provisory/tree/fix_simulations/examples).

At the moment i have tried to improve performance of the framework working to these simulations:
- Flockers
- WolfSheepGrass

To understand what i have tried check documentation of each experiment [here](https://github.com/Tonaion02/KrABMagaTirocinio_provisory/tree/fix_simulations/experiments_doc).

## EXECUTE EXPERIMENTS

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

