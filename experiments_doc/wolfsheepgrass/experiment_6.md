# EXPERIMENT 6

## WHAT I TRIED
With the previous experiment i have tried to obtain a speed-up parallelizing the operation of re-creating a field(DenseGrid).

I haven't noticed a notable speedup after this operation.
I have thinked that is because for race conditions about the bag for each cell, but during profiling i have noticed something strange:
**add_image_profiling**
From this trace we can see that all the threads is used only during the last part of recreating a field.
This suggest that there is another operation that takes the major part of the time.
How we can see the only other part of this method is the clearing of update field.
**add_code_block**

## NEW IMPLEMENTATION
Simple we will try to parallelize this operation.