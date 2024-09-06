use std::cell::RefCell;
use std::sync::RwLock;

use crate::engine::fields::grid_option::GridOption;
use crate::engine::location::Int2D;
use crate::engine::fields::field::Field;
use rand::Rng;
use bevy::prelude::Component;
use tui::widgets;





// T: TODO add a marker to this structure
#[derive(Component)]
pub struct DenseSingleValueGrid2D<T: Copy + Clone + PartialEq> {
    pub values: Vec<Option<T>>,
    pub width: i32,
    pub height: i32,
}


impl<T: Copy + Clone + PartialEq> DenseSingleValueGrid2D<T> {
    pub fn new(width: i32, height: i32) -> DenseSingleValueGrid2D<T> {
        DenseSingleValueGrid2D {
            values: vec![None; (width * height) as usize],
            width: width.abs(),
            height: height.abs(),
        }
    }



    pub fn apply_to_all_values<F>(&mut self, closure: F, option: GridOption)
    where
        F: Fn(&T) -> T,
    {
        for (i, elem) in self.values.iter_mut().enumerate() {

            if elem.is_none() {
                continue;
            }

            *elem = Some(closure(&elem.unwrap()));
        }
    }



    pub fn set_value_location(&mut self, value: T, loc: &Int2D) {
        let index = self.compute_index(loc);
        self.values[index] = Some(value);
    }


    pub fn get_value(&self, loc: &Int2D) -> Option<T> {
        return self.values[self.compute_index(loc)];
    }

    // T: TODO substitute with a macro
    fn compute_index(&self, loc: &Int2D) -> usize {
        return ((loc.y * self.width) + loc.x) as usize
    }
}