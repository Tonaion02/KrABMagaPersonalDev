# EXPERIMENT 4

## WHAT I TRIED
For this experiment i tried another way to modelize the interaction beetween sheeps and wolfs. 
This is not really necessary for this simulation. In this simulation is not necessary that a wolf specifically eat a specifically sheep. In other simulation where is important that, we must gave the possibility to do very rapidly this.

## TRIVIAL IMPLEMENTATION
In the case of this simulation, the implementation that i have tried in the experiment_3 may be enough.
In theory we only know that each wolf try to eat a sheep that is in the same location. So we can directly count the number of wolfs for location, the number of sheeps for location, and the number of sheep that has be eaten/wolves that gain energy is equal to the minimum of the two counter for each location.

This method cannot be applied to more complex simulations. In many simulations we need for example that an agents choose the other agents with some euristic or logic.
In this experiment i'm trying to find a solution that can work also in these other situasions.

## NEW IMPLEMENTATION
In this case 