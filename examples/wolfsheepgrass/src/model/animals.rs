use krabmaga::engine::location::Int2D;
use krabmaga::engine::location::Real2D;
// T: TODO check if we can solve this strange error with import
// T: of Component and get rid of this ugly 
// T: and apparently useless import of bevy_ecs
// T: see that for a description of the problem:
// T: https://github.com/bevyengine/bevy/issues/3659
use krabmaga::engine::bevy_ecs as bevy_ecs;
use krabmaga::engine::Component;





// T: TODO check if it is the best way to store data about agents.
// T: probably we can find a more conveniente way to partitionate 
// T: from the multithreading point of view......
#[derive(Component, Copy, Clone)]
pub struct Wolf {
    pub id: u32,
    pub loc: Int2D,
    pub last: Option<Int2D>,
    pub energy: f64,
    pub gain_energy: f64,
    pub prod_reproduction: f64,
}

#[derive(Component, Copy, Clone)]
pub struct Sheep {
    pub id: u32,
    pub loc: Int2D,
    pub last: Option<Int2D>,
    pub energy: f64,
    pub gain_energy: f64,
    pub prod_reproduction: f64,
}

// T: this component is used to save the last location of the animals
#[derive(Component, Copy, Clone)]
pub struct LastLocation(pub Option<Int2D>);

// T: this component contains the updated location of these agents
#[derive(Component, Copy, Clone)]
pub struct Location(pub Int2D);