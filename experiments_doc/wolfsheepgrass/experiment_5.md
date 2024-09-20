# EXPERIMENT 5

## WHAT I TRIED
With the previous experiment i have obtained a speed-up and a manner to handle complex interaction within multiple agents.

Before this experiment i didn't tried to parallelize many operations.
Some sequential operations are:
- Update of dense grids
- Adding/Removing entities from ECS storing
- Sheep eating grass

In this experiment i tried to parallelize the update of dense grids. 

## PREVIOUS TRIVIAL IMPLEMENTATION
In the previous and trivial implementation of this operation, i rebuild from zero the entire grid with a sequential approach.

So we clear all the bags of the dense grid. The clear operation doesn't allocate memory, we just reset some index.
Then each agents is inserted in a bag of the grid. 

Rebuild these dense grids cannot be easily parallelized because we need for each agents of the pool to insert(and so modify) the grid. This is a race condition among the grid for each agent.

## NEW IMPLEMENTATION
We can't easily solve the problem putting a lock on the grid. Because each agent make a race condition among the grid.

If we see the problem from another prospective, we can say that each agent make a race condition among a bag of the grid. We don't modify the grid or the vector where the bags reside. For each agent we only modify one bag.

The idea is to put a lock on each bag. With this technique we can lock only the bag where the agent must be inserted. With this technique we easily reduce the number of threads that wait other threads to release the locks.