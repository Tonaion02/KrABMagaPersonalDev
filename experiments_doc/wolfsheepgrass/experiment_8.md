# EXPERIMENT 8

## WHAT IS THE PROBLEM?
From Experiment 6 the last bottleneck we had is the spawning/despawning of agents in the simulation.

Even if we can easily write parallel code in a simulation where we must spawn/despawn agents witht the Commands pattern and the use of [ParallelCommands](https://docs.rs/bevy/latest/bevy/prelude/struct.ParallelCommands.html) we encountered the problem that the commands are solved in a sequential manner.
We are now experimenting some new way to accomplish this task.

With Experiment 7 we tried the more simple solution that comes to mind.
We leaved unmodified the despawning logic, and we continued to use ParallelCommands for it, but we tried to save the data of agents to spawn in a [Parallel](https://docs.rs/bevy/latest/bevy/utils/struct.Parallel.html) and then use it for [spawn_batch](https://docs.rs/bevy/latest/bevy/ecs/prelude/struct.Commands.html#method.spawn_batch). spawn_batch as how it is said in this [issue]() must be save some times.

The results aren't so incredible. Probably the best gain is from not use ParallelCommands. In fact the major time we gain is from step.

## NEW IMPLEMENTATION
What we want to do now, is starting implementing a new particular system, the Cimitery System that reppresent the idea to serve to the end-user a more fast method to spawn and despawn agents.
The idea for the implementation is:
- Sign in a Parallel the agent to spawn during the normal step.
- Sign in another Parallel the agent to delete during the normal step.
- In the CimiterySystem we want to:
    - Write on all the agent to delete the data of an agent to spawn(so we spawned and deleted the the same time).
    - If the number of agents to spawn is greater than the agent to delete, spawn more agents with the convenient technique of spawn_batch.
    - If the number of agents to despawn is greater than the agent to spawn, despawn the remaining agents.

This system born from the idea that probably the best way(from performance point of view) in Bevy to spawn/despawn is not despawn and re-spawn agents of the same type.  