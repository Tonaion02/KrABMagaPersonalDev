use bevy::prelude::{Component, Query};

use crate::engine::components::double_buffer::{DBRead, DBWrite};
use crate::engine::components::double_buffer::{DBClonableRead, DBClonableWrite};

pub fn double_buffer_sync<T: Component + Copy + Send>(
    mut query: Query<(&mut DBRead<T>, &DBWrite<T>)>,
) {
    //T: probably is not necessarily to parallelize this operation, a simple copy
    //T: that take really a little amount of time compared to other operations
    // TODO parallelize
    /*query.par_for_each_mut(50000/8, |(mut read, write)| {
        read.0 = write.0;
    });*/
    for (mut read, write) in &mut query {
        read.0 = write.0;
    }
}

// T: Added by me
// T: It is necessary to make double buffering along something like datastructures
pub fn double_buffer_clonable_sync<T: Component + Clone>(
    mut query: Query<(&mut DBClonableRead<T>, &DBClonableWrite<T>)>,
) {
    for( mut read, write) in &mut query {
        read.0 = write.0.clone();
    }
}