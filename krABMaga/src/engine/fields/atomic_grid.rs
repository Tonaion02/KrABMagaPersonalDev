use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, AtomicU32};

use bevy::prelude::default;

use crate::engine::location::Int2D;

use crate::engine::Component;


#[derive(Component)]
pub struct AtomicGrid2D<M:Sized> {

    pub values: Vec<AtomicU32>,
    pub width: i32,
    pub height: i32,

    phantom: PhantomData<M>,
}

impl<M:Sized> AtomicGrid2D<M> {
    
    pub fn new(default_value: u32, width: i32, height: i32) -> AtomicGrid2D<M> {
        let mut grid = AtomicGrid2D {
            values: Vec::<AtomicU32>::new(),
            width: width,
            height: height,

            phantom: PhantomData,
        };

        for i in 0..(height * width) {
            grid.values.push(AtomicU32::new(default_value));
        }

        grid
    }  

    pub fn get_ref_counter(&self, loc: &Int2D) -> &AtomicU32 {
        return &self.values[self.compute_index(loc)];
    }

    // T: TODO substitute with a macro
    fn compute_index(&self, loc: &Int2D) -> usize {
        return ((loc.y * self.width) + loc.x) as usize
    }
}