use bevy::prelude::*;

/// Automatically create two copies for the data provided. After each step, the write buffer is fully applied on the read one.
#[derive(Bundle)]
pub struct DoubleBuffered<T: Component + Copy + Send> {
    pub read: DBRead<T>,
    pub write: DBWrite<T>,
}

impl<T: Component + Copy + Send> DoubleBuffered<T> {
    pub fn new(component: T) -> DoubleBuffered<T> {
        DoubleBuffered {
            read: DBRead(component),
            write: DBWrite(component),
        }
    }
}

//T: problem, probably there isn't a bundle version for reading and writing, probably Bundle is used only
//T: for insertions.
// TODO simplify querying those structs by offering a complete read+write bundle and a read one? Would the user ever need to only query the write buffer?
#[derive(Component)]
pub struct DBRead<T: Component + Copy + Send>(pub T);

#[derive(Component)]
pub struct DBWrite<T: Component + Copy + Send>(pub T);





#[derive(Bundle)]
pub struct DoubleBufferedDataStructure<T: Component + Clone> {
    pub read: DBClonableRead<T>,
    pub write: DBClonableWrite<T>,
}

// T: TODO verify is is more conveniente to use the value
// T: probably is moved anyway.
impl<T: Component + Clone> DoubleBufferedDataStructure<T> {
    pub fn new(component: T) -> DoubleBufferedDataStructure<T> {
        DoubleBufferedDataStructure {
            read: DBClonableRead(component.clone()),
            write: DBClonableWrite(component.clone()),
        }
    }
}

#[derive(Component)]
pub struct DBClonableRead<T: Component + Clone>(pub T);

#[derive(Component)]
pub struct DBClonableWrite<T: Component + Clone>(pub T);