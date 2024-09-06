use std::hash::Hash;
use std::option::Option;

use crate::engine::location::Int2D;





// T: A dense grid where each cell is a bag of object
// T: this version doesn't implement double buffering
// T: TODO add all the methods that are missing from 
// T: this implementation
pub struct DenseBagGrid2D<O: Eq + Hash + Clone + Copy> {
    pub bags: Vec<Vec<O>>,
    pub width: i32,
    pub height: i32,
}

impl<O: Eq + Hash + Clone + Copy> DenseBagGrid2D<O> {

    pub fn new(width: i32, height: i32) -> DenseBagGrid2D<O> {
        DenseBagGrid2D {
            bags: std::iter::repeat_with(Vec::new).take((width * height) as usize).collect(),
            width: width,
            height: height,
        }
    }    

    pub fn get_objects(&self, loc: &Int2D) -> Option<Vec<O>> {
        let mut cloned_bag = Vec::new();
        let index = self.compute_index(loc);

        if self.bags[index].is_empty() {
            None
        } else {
            // T: TODO check if this has some sense
            // T: I think that Vec implement Clone function
            for elem in &self.bags[index] {
                cloned_bag.push(*elem);
            }
            Some(cloned_bag)
        }
    }

    pub fn remove_object_location(&mut self, object: O, loc: &Int2D) {

        let index = self.compute_index(loc);

        if (self.bags[index].is_empty()) {
            self.bags[index].retain(|&obj| obj != object);
        }

    }

    // T: TODO substitute with a macro
    fn compute_index(&self, loc: &Int2D) -> usize {
        return ((loc.y * self.width) + loc.x) as usize
    }
}