# EXPERIMENT 4

## WHAT I TRIED
For this experiment i tried another way to modelize the interaction beetween sheeps and wolfs. 
This is not really necessary for this simulation. In this simulation is not necessary that a wolf specifically eat a specifically sheep. In other simulation where is important that, we must gave the possibility to do very rapidly this.

## TRIVIAL IMPLEMENTATION
In the case of this simulation, the implementation that i have tried in the experiment_3 may be enough.
In theory we only know that each wolf try to eat a sheep that is in the same location. So we can directly count the number of wolfs for location, the number of sheeps for location, and the number of sheep that has be eaten/wolves that gain energy is equal to the minimum of the two counter for each location.

This method cannot be applied to more complex simulations. In many simulations we need for example that an agents choose the other agents with some euristic or logic.
In this experiment i'm trying to find a solution that can work also in these other more complex simulations.

## NEW IMPLEMENTATION
I mantained the constraint that every agents interact only with the agents in the same location.
The approach is to loop in parallel among the two DenseGrid(One for the sheep and another for the wolves). So for each bag of wolves and sheep that reside in the same location i need for each one wolves to select an alive sheep(if there is) to kill. Each wolves that kill a sheep, gain energy from that.
I haven't used this technique in the previous experiments because there is a problem. To apply this tecnique i need at least to modify the data of the wolves(the energy of each wolves that has eatean a sheep).

I canno't use a mutable query object from Bevy to do that. Because mutable object cannot be used in parallel context like shared object.

I am sure that in this parallel iteration i'm going to modify different entities each time. I am sure of that because the entities are unique in the DenseGrids.
So i only need a method to indicate to the Rust compiler that i'm sure that my code doesn't access the same entities from different threads.

To do that i can use some lock. With lock i give the assurance to the compiler the assurance that multiple threads cannot access to the same memory location at the same time.

If i put a lock directly on mutable query object, the code became really slow. This because there is a race condition on an object(the query) that must be used every time. This lead to the fact that every threads lock the same object. The code can be slower than a sequential version of the code(I haven't tried this approach cause i don't need to test to know that is really really slow).

The idea is to put lock directly on the data of the agents that must be accessed in parallel context. There isn't really race condition, only one thread at time only one time acquire the lock and release the lock. This mutex is only necessary to Rust to be sure that multiple threads cannot access to the same memory location at the same time, even if we are sure of that for a constraint of the problem, we must give to rust this assurance.