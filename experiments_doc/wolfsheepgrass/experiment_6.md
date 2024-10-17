# EXPERIMENT 6

## WHAT I TRIED
With the previous experiment i have tried to obtain a speed-up parallelizing the operation of re-creating a field(DenseGrid).

I haven't noticed a notable speedup after this operation.
I have thinked that is because for race conditions about the bag for each cell, but during profiling i have noticed something strange:
<img title="profiling_image_exp_6" alt="profiling_image_exp_6" src="profiling_image_exp_6.PNG">
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

## NEW IMPLEMENTATION
Simple we will try to parallelize this operation.