use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::Mutex;

use bevy::prelude::default;

use crate::engine::location::Int2D;

use crate::engine::Component;


#[derive(Component)]
pub struct AtomicGrid2D<M:Sized> {

    pub values: Vec<Arc<Mutex<u32>>>,
    pub width: i32,
    pub height: i32,

    phantom: PhantomData<M>,
}

impl<M:Sized> AtomicGrid2D<M> {
    
    pub fn new(default_value: u32, width: i32, height: i32) -> AtomicGrid2D<M> {
        let mut grid = AtomicGrid2D {
            values: Vec::<Arc<Mutex<u32>>>::new(),
            width: width,
            height: height,

            phantom: PhantomData,
        };

        for i in 0..(height * width) {
            grid.values.push(Arc::new(Mutex::new(default_value)));
        }

        grid
    }  

    // pub fn get_ref_counter(&self, loc: &Int2D) -> &AtomicU32 {
    //     return &self.values[self.compute_index(loc)];
    // }

    pub fn get_atomic_counter(&self, loc: &Int2D) -> Arc<Mutex<u32>> {
        return Arc::clone(&self.values[self.compute_index(loc)])
    }

    // T: TODO substitute with a macro
    fn compute_index(&self, loc: &Int2D) -> usize {
        return ((loc.y * self.width) + loc.x) as usize
    }
}