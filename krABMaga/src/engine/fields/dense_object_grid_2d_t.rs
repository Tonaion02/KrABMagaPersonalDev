use std::hash::Hash;
use std::option::Option;
use std::marker::PhantomData;

use crate::engine::location::Int2D;

use crate::engine::Entity;
use crate::engine::Component;





// T: A dense grid where each cell is a bag of object
// T: this version doesn't implement double buffering
// T: TODO add all the methods that are missing from 
// T: this implementation
#[derive(Component)]
pub struct DenseBagGrid2D<O: Eq + Hash + Clone + Copy, M: Sized> {
    pub bags: Vec<Vec<O>>,
    pub width: i32,
    pub height: i32,

    phantom: PhantomData<M>,
}

impl<O: Eq + Hash + Clone + Copy, M: Sized> DenseBagGrid2D<O, M> {

    pub fn new(width: i32, height: i32) -> DenseBagGrid2D<O, M> {
        DenseBagGrid2D {
            bags: std::iter::repeat_with(Vec::new).take((width * height) as usize).collect(),
            width: width,
            height: height,

            phantom: PhantomData,
        }
    }   

    pub fn set_object_location(&mut self, object: O, loc: &Int2D) {
        let index = self.compute_index(loc);
        
        if !self.bags[index].is_empty() {
            self.bags[index].retain(|&obj| obj != object);
        }

        self.bags[index].push(object);
    }

    // T: TODO Refactor the name with a more indicative way
    // T: this particular method permits to insert in a bag
    // T: one object without the overhead of the retain operation
    pub fn set_object(&mut self, object: O, loc: &Int2D) {
        let index = self.compute_index(loc);
        self.bags[index].push(object);
    }

    pub fn clear(&mut self) {
        self.bags.iter_mut().for_each(|mut vec|{ vec.clear(); })
    }

    pub fn get_objects(&self, loc: &Int2D) -> Option<Vec<O>> {
        let mut cloned_bag = Vec::new();
        let index = self.compute_index(loc);

        if self.bags[index].is_empty() {
            None
        } else {
            // T: TODO check if this has some sense
            // T: I think that Vec implement Clone function
            // T: taken from the original implementation
            for elem in &self.bags[index] {
                cloned_bag.push(*elem);
            }
            Some(cloned_bag)
        }
    }

    // T: This method is useful when you want to put 
    // T: the contents of a bag in an already allocated
    // T: buffer.
    pub fn get_object_already_allocation(&self, loc: &Int2D, mut buffer: &mut Vec::<O>) {

        let index = self.compute_index(loc);

        // T: TODO check if this has some sense
        // T: I think that Vec implement Clone function
        // T: taken from the original implementation
        for elem in &self.bags[index] {
            buffer.push(*elem);
        }
    }

    // T: This function is written to evitate to allocate each time 
    // T: to recopy a bag when you need to modify it.
    pub fn get_ref_mut_bag(&mut self, loc: &Int2D) -> &mut Vec<O> {
        let index = self.compute_index(loc);
        return &mut self.bags[index];
    }

    // T: This function is written to evitate to allocate each time 
    // T: to recopy a bag when you need to read it
    pub fn get_ref_bag(&self, loc: &Int2D) -> &Vec<O> {
        let index = self.compute_index(loc);
        return & self.bags[index];
    }

    pub fn remove_object_location(&mut self, object: O, loc: &Int2D) {

        let index = self.compute_index(loc);

        if (! self.bags[index].is_empty()) {
            self.bags[index].retain(|&obj| obj != object);
        }

    }

    pub fn remove_object_with_index(&mut self, loc: &Int2D, index: usize) {
        let index_bag = self.compute_index(loc);
        if ! self.bags[index_bag].is_empty() {
            self.bags[index_bag].remove(index);
        }
    }

    // T: TODO substitute with a macro
    fn compute_index(&self, loc: &Int2D) -> usize {
        return ((loc.y * self.width) + loc.x) as usize
    }
}