# EXPERIMENT 3

## WHAT I TRIED
For this experiment i tried to parallelize a specific set of operations of this simulation.
At certain point of the step the wolves eats sheeps.
Then i tried to parallelize the updating of the grass.

## TRIVIAL IMPLEMENTATION OF WOLVES EAT SHEEPS
The most trivial implementation is:
- query for sheeps(mutable)
- query for wolves(mutable) and wolf's location
- retrieve the field of the sheeps(mutable)
- for each wolf check in the field if there is a sheep in the wolf's location, if it is true:
    - gain energy to wolf
    - remove the sheep from the field

In this case we need to modify the field to stop other wolves from eating death sheeps. So this operation is really difficult to implement in parallel for the need to notify to other wolves that a certain sheep is not valid to eat.

## NEW IMPLEMENTATION OF WOLVES EAT SHEEPS
I understood that the unique things i care is how many sheeps die and how many wolves gain energy for each location(obviously the same number).

What i tried is to compute in a grid the numbers of wolves for each location with an atomic counter for each location(cell of the grid).

After that i runned in parallel a system on the sheeps' field and the grid, and i used these to despawn the correct amount of sheeps from the grid.

Then i used the number of wolves for each cell, a classic field for the sheeps and a query on the wolves and their location to gain energy for each wolf.
In parallel, for each wolf i compute the minimum beetween the number of sheeps in that location and the number of wolves, decrease the minimum and assign it like new counter in the grid, then i gain energy to wolf if counter is greater than zero.

## NEW IMPLEMENTATION OF GROWING GRASS
I simply used a parallel iterator from [rayon](https://docs.rs/rayon/latest/rayon/) to parallelize the process of updating of the grass.
I need to test if this is convenient in this case, we are talking about a simply vector updating that can easily be performed in easily manners from the compiler.