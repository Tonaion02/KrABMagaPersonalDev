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
use crate::engine::location::Real2D;

use crate::engine::Entity;
use crate::engine::Component;





//=====================================================================================================
//-----------------------------------------------------------------------------------------------------
// PARALLEL DENSE BAG GRID
//-----------------------------------------------------------------------------------------------------
// T: Parallel Dense Bag Grid used in the flocker's experiment number 1
//
// T: This version of Parallel Dense Bag Grid is based on the version
// T: of WolfSheepGrass's experiment 6. 
// T: The difference in this case is that we want to implement all the method
// T: necessary to retrieve the neighborhood of entities.
//-----------------------------------------------------------------------------------------------------
#[derive(Component)]
pub struct ParDenseBagGrid2D_flockers_exp_1<O: Eq + Hash + Clone + Copy + Send, M: Sized> {

    pub bags: Vec<RwLock<Vec<O>>>,
    pub width: f32,
    pub height: f32,

    pub discretization: f32,
    toroidal: bool,

    phantom: PhantomData<M>,
}

impl<O: Eq + Hash + Clone + Copy + Send, M: Sized> ParDenseBagGrid2D_flockers_exp_1<O, M> {

    // T: New methods (START)
    pub fn get_neighbors_within_relax_distance(&self, loc: Real2D, dist: f32) -> Vec<O> {
        //let density = ((self.width * self.height) as usize) / (self)
        
        // T: TODO try to reimplement the logic of determination of
        // T: capacity of neighbors.
        let mut neighbors: Vec<O> = Vec::with_capacity(500);

        if dist <= 0.0 {
            return neighbors;
        }

        let disc_dist = (dist / self.discretization).floor() as i32;
        let disc_loc = self.discretize(&loc);
        let max_x = (self.width / self.discretization).ceil() as i32;
        let max_y = (self.height / self.discretization).ceil() as i32;

        let mut min_i = disc_loc.x - disc_dist;
        let mut max_i = disc_loc.x + disc_dist;
        let mut min_j = disc_loc.y - disc_dist;
        let mut max_j = disc_loc.y + disc_dist;

        if self.toroidal {
            min_i = std::cmp::max(0, min_i);
            max_i = std::cmp::min(max_i, max_x - 1);
            min_j = std::cmp::max(0, min_j);
            max_j = std::cmp::min(max_j, max_y - 1);
        }

        for i in min_i..max_i + 1 {
            for j in min_j..max_j + 1 {
                let bag_id = Int2D {
                    x: self.t_transform(i, max_x),
                    y: self.t_transform(j, max_y),
                };
                // T: commented by me, isn't no more neccesary this code
                // let vector = match self.fbag.get(&bag_id) {
                //     Some(i) => i,
                //     None => continue,
                // };
                
                let index = self.compute_index(&bag_id);

                // T: Retrieve the correct bag and make a lock on read
                let bag = self.bags[index].read().unwrap();

                // T: TODO understand if this code makes copy
                for elem in bag.iter() {
                    neighbors.push(*elem);
                }

                // T: TODO check if this is necessary
                std::mem::drop(bag);
            }
        }

        neighbors
    }


    // T: NOTES i can't understand what function does
    // T: but like a good programmer, this isn't a problem
    // T: for me.
    // T: TODO move this method out or make it static or make it macro
    fn t_transform(&self, n: i32, size: i32) -> i32 {
        if n >= 0 {
            n % size
        } else {
            (n % size) + size
        }
    }

    // T: NOTES who know this method what does
    // T: TODO move this method out or make it static or make it macro
    pub fn discretize(&self, loc: &Real2D) -> Int2D {
        let x_floor = (loc.x / self.discretization).floor();
        let x_floor = x_floor as i32;

        let y_floor = (loc.y / self.discretization).floor();
        let y_floor = y_floor as i32;

        Int2D {
            x: x_floor,
            y: y_floor,
        }
    }

    // T: New methods (END)





    pub fn new(width: i32, height: i32, discretization: f32, toroidal: bool) -> ParDenseBagGrid2D_flockers_exp_1<O, M> {

        let mut bags = Vec::<RwLock<Vec<O>>>::new();
        for i in 0..(width * height) as usize {
            bags.push(RwLock::new(Vec::new()));
        }

        ParDenseBagGrid2D_flockers_exp_1 {
            width: width as f32,
            height: height as f32,

            bags: bags,
            
            discretization: discretization,
            toroidal: toroidal,

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

    pub fn clear(&mut self) {
        self.bags.par_iter_mut().for_each(|(bag)|{
            bag.write().expect("Missing bags").clear();
        });
    }

    fn compute_index(&self, loc: &Int2D) -> usize {
        return ((loc.y * self.width as i32) + loc.x) as usize
    }

}
//=====================================================================================================