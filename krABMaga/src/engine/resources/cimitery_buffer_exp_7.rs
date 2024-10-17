use std::vec::Vec;
use std::hash::Hash;
use std::cell::RefCell;

use std::marker::PhantomData;

use crate::engine::Component;





//=====================================================================================================
//-----------------------------------------------------------------------------------------------------
// T: The buffer used in parallel context to keep data that must be used during
// T: a deferred spawn/delete phase of the step.
//-----------------------------------------------------------------------------------------------------
// T:                                  <DESCRIBING MY IDEAS>
// T: What we want to achieve with this data structures is the possibility to 
// T: add elements in a parallel context.
// T: An idea to make this in a very rapidly manner is to use the internal mutability
// T: pattern, so we don't have necessity of a mutable reference to push data directly
// T: in one of the buffers.
//-----------------------------------------------------------------------------------------------------
#[derive(Component)]
pub struct CimiteryBufferExp7<O: Eq + Hash + Clone + Copy + Send, M: Sized> {

    pub buffers: RefCell<Vec<Vec<O>>>,

    phantom: PhantomData<M>,
}

impl<O: Eq + Hash + Clone + Copy + Send, M: Sized> CimiteryBufferExp7<O, M> {

    pub fn new(num_threads: usize) -> CimiteryBufferExp7<O, M>{
        
        let mut buffers = Vec::new();
        buffers.resize(num_threads, Vec::new());

        //println!("buffers capacity: {}", buffers.capacity());

        CimiteryBufferExp7 {
            buffers: RefCell::new(buffers),

            phantom: PhantomData,
        }
    }

    pub fn push(self: CimiteryBufferExp7<O, M>, object: O) {

        let id_thread = 0;

        let mut mut_buffers = self.buffers.borrow_mut();
        mut_buffers[id_thread].push(object);
    }
}
//=====================================================================================================