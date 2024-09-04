// T: Component that represents an Animal(either a sheep or a wolf)
#[derive(Component, Copy, Clone)]
pub struct Animal {
    // T: identifiers for animal
    // T: TODO check if we need this variable anymore
    // T: TODO check if we need to enumerate in different manner sheeps and
    // T: wolves.
    pub id: u32,
    // T: actual location of the animal
    // T: TODO check if this variable is placed in the Animal or Out of that
    pub loc: Int2D,
    // T: last location of the animal
    // T: TODO check if this variable is placed in the Animal or Out of that
    pub last: Option<Int2D>,
    // T: current remaining energy
    pub energy: f64,
    // T: Gain of energy for each animals
    // T: TODO check if we can define that directly like
    // T: a constant in the program.
    pub gain_energy: f64,
    // T: probability of reproduction
    pub prob_reproduction: f64,
}

// T: I use a different component to mark that an animal is alive
// T: to have directly a buffer with all the animal that are alive
// T: and evitate an useless check
// T: TODO check if we really need this or we can direclty delete the
// T: dead animals.......
// T: TODO check if the change of storage is effectively
#[derive(Componet, Copy, Clone)]
#[component(storage="SparseSet")]
pub struct Alive;

// T: TODO check if we really need this component, probably we can get rid
// T: out of that.
#[derive(Component, Copy, Clone)]
#[component(storage="SparseSet")]
pub struct Dead;

// T: TODO check if we need these two structures
#[derive(Component, Copy, Clone)]
#[component(storage="SparseSet")]
pub struct Wolf;

#[derive(Component, Copy, Clone)]
#[component(storage="SparseSet")]
pub struct Sheep;