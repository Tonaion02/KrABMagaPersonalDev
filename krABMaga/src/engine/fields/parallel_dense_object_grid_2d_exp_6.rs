use std::hash::Hash;
use std::option::Option;
use std::marker::PhantomData;

use std::mem::swap;
use std::sync::RwLock;
use std::sync::RwLockWriteGuard;
use std::sync::RwLockReadGuard;

// T: importing rayon (START)
extern crate rayon;
use rayon::iter::IntoParallelRefMutIterator;
use rayon::iter::ParallelIterator;
// use rayon::iter::ParallelIterator;
// use rayon::iter::IntoParallelRefIterator;
// use rayon::iter::IndexedParallelIterator;
// T: importing rayon (END)

use crate::engine::location::Int2D;

use crate::engine::Entity;
use crate::engine::Component;





//=====================================================================================================
//-----------------------------------------------------------------------------------------------------
// T: Parallel Dense Bag Grid used in the experiment 6
// T: A dense grid where each cell is a bag of objects
// T: wrapped around an RwLock.
// T: This version doesn't implement double buffering.
// T: WARNING: The difference between this version of grid and the version
// T: of experiment 5 is the fact that this version has a clear that works
// T: with multithreading.
//-----------------------------------------------------------------------------------------------------
#[derive(Component)]
pub struct ParDenseBagGrid2D_exp_6<O: Eq + Hash + Clone + Copy + Send, M: Sized> {

    pub bags: Vec<RwLock<Vec<O>>>,
    pub width: i32,
    pub height: i32,

    phantom: PhantomData<M>,
}

impl<O: Eq + Hash + Clone + Copy + Send, M: Sized> ParDenseBagGrid2D_exp_6<O, M> {

    pub fn new(width: i32, height: i32) -> ParDenseBagGrid2D_exp_6<O, M> {

        let mut bags = Vec::<RwLock<Vec<O>>>::new();
        for i in 0..(width * height) as usize {
            bags.push(RwLock::new(Vec::new()));
        }

        ParDenseBagGrid2D_exp_6 {
            width: width,
            height: height,

            bags: bags,

            phantom: PhantomData,
        }
    }

    pub fn get_write_bag<'a>(&'a self, loc: &Int2D) -> RwLockWriteGuard<'a, Vec<O>> {
        let index = self.compute_index(loc);

        self.bags[index].write().unwrap()
    }

    pub fn get_read_bag<'a>(&'a self, loc: &Int2D) -> RwLockReadGuard<'a, Vec<O>> {
        let index = self.compute_index(loc);

        self.bags[index].read().unwrap()
    }

    pub fn push_object_location(&mut self, object: O, loc: &Int2D) {
        let index = self.compute_index(loc);

        self.bags[index].write().unwrap().push(object);
    }

    // T: this method works in multithreading(difference with experiment_5's version) 
    pub fn clear(&mut self) {
        // T: TEMP
        // for bag in &mut self.bags {
        //     let mut bad_bind = bag.write().unwrap();
        //     bad_bind.clear();
        // }

        self.bags.par_iter_mut().for_each(|(bag)|{
            bag.write().expect("Missing bags").clear();
        });
    }

    fn compute_index(&self, loc: &Int2D) -> usize {
        return ((loc.y * self.width) + loc.x) as usize
    }
}
//=====================================================================================================