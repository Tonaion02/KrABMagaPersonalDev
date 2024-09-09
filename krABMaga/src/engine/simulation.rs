use core::num;
use std::thread;
use std::time::Duration;

use bevy::core::TaskPoolThreadAssignmentPolicy;
use bevy::log::LogPlugin;
use bevy::prelude::*;

use crate::engine::location::Real2D;

use crate::engine::fields::field_2d::{update_field, Field2D};
use crate::engine::rng::RNG;
use crate::engine::systems::double_buffer_sync::double_buffer_sync;
use crate::engine::systems::simulation_descriptor_update::simulation_descriptor_update_system;

use crate::engine::resources::simulation_descriptor::SimulationDescriptorT;

use super::resources::simulation_descriptor;





// TEMP
// T: made public for wolfsheepgrass(temporary)
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum SimulationSet {
    BeforeStep,
    Step,
    AfterStep,
}

pub struct Simulation {
    // TEMP: temporary modified
    //pub(crate) app: App,
    pub app: App,
    steps: Option<u32>,
    num_threads: usize,
}

impl Simulation {
    pub fn build() -> Self {

        let mut app = App::new();

        // T: OLD, it make working in the beginning
        // self.app.add_plugins(DefaultPlugins.set(TaskPoolPlugin {
        //     task_pool_options: TaskPoolOptions {
        //         // Assign all threads to compute
        //         compute: TaskPoolThreadAssignmentPolicy {
        //             // set the minimum # of compute threads
        //             // to the total number of available threads
        //             min_threads: 4,
        //             max_threads: 4, // unlimited max threads
        //             percent: 1.0,             // this value is irrelevant in this case
        //         },
        //         ..default()
        //     },
        // }));
        // T: OLD, it make working in the beginning

        #[cfg(any(feature = "trace_tracy", feature = "visualization"))]
        app.add_plugins(LogPlugin::default());

        // T: trying to move this in with_num_threads
        // app.add_plugins(TaskPoolPlugin::default());
            
        app.configure_sets(
            Update,
            (
                SimulationSet::BeforeStep,
                SimulationSet::Step,
                SimulationSet::AfterStep,
            )
                .chain(),
        );

        //T: make SimulationDescriptor a Resource
        let simulation_descriptor = SimulationDescriptorT::default();
        app.insert_resource(simulation_descriptor);
        //T: make SimulationDescriptor a Resource

        //T: In case feature visualization not is defined
        #[cfg(not(feature = "visualization"))]
        app.add_systems(
                Update,
                (simulation_descriptor_update_system,).in_set(SimulationSet::BeforeStep),
        );
        //T: In case feature visualization not is defined

        //T: In case feature visualization is defined
        //T: In fact in the case where we define visualization we need to
        //T: run engine_config_update and step in FixedPreUpdate so we can
        //T: decide how much time in a seconds we want to run this systems 
        #[cfg(feature = "visualization")]
        app.add_systems(
            FixedPreUpdate,
            (simulation_descriptor_update_system).in_set(SimulationSet::BeforeStep).run_if(Simulation::is_not_paused),
        );
        //T: In case feature visualization is defined

        Self { app, steps: None, num_threads: 1 }
    }

    pub fn with_simulation_dim(mut self, simulation_dim: Real2D) -> Self {

        // T: can't panick, we inserted resource during build...
        self.app.world.get_resource_mut::<SimulationDescriptorT>().unwrap().simulation_dim = simulation_dim;

        self
    }

    pub fn with_num_threads(mut self, num_threads: usize) -> Self {
        self.app.add_plugins(TaskPoolPlugin {
            task_pool_options: TaskPoolOptions {
                // Assign all threads to compute
                compute: TaskPoolThreadAssignmentPolicy {
                    // set the minimum # of compute threads
                    // to the total number of available threads
                    min_threads: num_threads,
                    max_threads: num_threads, // unlimited max threads
                    percent: 1.0,             // this value is irrelevant in this case
                },
                ..default()
            },
        });

        // T: can't panick, we inserted resource during build...
        self.app.world.get_resource_mut::<SimulationDescriptorT>().unwrap().num_threads = num_threads;

        // T: TODO remove, cause it's now useless
        self.num_threads = num_threads;

        self
    }

    // TODO expose a macro to wrap a fn describing the step of one agent and transform it in a system that cycles all agents? This is probably the worst aspect of the refactor, the step signature can easily get too complex to read.
    pub fn register_step_handler<Params>(
        mut self,
        step_handler: impl IntoSystemConfigs<Params>,
    ) -> Self {
        #[cfg(not(feature = "visualization"))]
        self.app.add_systems(Update, (step_handler,).in_set(SimulationSet::Step));

        #[cfg(feature = "visualization")]
        self.app.add_systems(FixedPreUpdate, (step_handler,).in_set(SimulationSet::Step).run_if(Simulation::is_not_paused));
        
        self
    }

    // T: Method to register a system for init of world
    pub fn register_init_world<Params>(mut self, init_world: impl IntoSystemConfigs<Params>) -> Self {
        self.app.add_systems(Startup, init_world);

        self
    }

    // TODO figure out a way to automatically register double buffers
    pub fn register_double_buffer<T: Component + Copy + Send>(mut self) -> Self {
        self.app.add_systems(
            Update,
            (double_buffer_sync::<T>,).in_set(SimulationSet::AfterStep),
        );
        
        self
    }

    pub fn with_steps(mut self, steps: u32) -> Self {
        // T: TODO remove, cause it's now useless
        self.steps = Some(steps);

        // T: can't panick, we inserted resource during build...
        self.app.world.get_resource_mut::<SimulationDescriptorT>().unwrap().steps = Some(steps);

        self
    }

    //T: commented cause we probably we don't need this anymore
    // TODO specify this is required (SimulationBuilder with validation, which generates a Simulation on build()?)
    // pub fn with_engine_configuration(mut self, config: EngineConfiguration) -> Self {
    //     self.app.insert_resource(config);

    //     self
    // }

    pub fn with_rng(mut self, seed: u64) -> Self {
        let rng = RNG::new(seed, 0);
        self.app.insert_resource(rng);

        self
    }

    pub fn add_field(&mut self, field: Field2D<Entity>) -> &mut Simulation {
        self.app.world.spawn((field,));
        self.app
            .add_systems(Update, (update_field,).in_set(SimulationSet::BeforeStep));

        self
    }

    pub fn run(mut self) {
        match self.steps {
            Some(steps) => {
                for _ in 0..steps {
                    self.app.update(); // TODO better approach? This seems to work fine but the example suggests a dedicated scheduler
                }
            }
            None => {
                println!("Running");
                self.app.run();
            }
        }
    }

    pub(crate) fn spawn_agent(&mut self) -> EntityWorldMut {
        self.app.world.spawn(())
    }





    // T: static function/system that works like run conditions(RC) for the simulation
    // T: this function/system determines if the simulation is paused
    pub(crate) fn is_not_paused(simulation_descriptor: Res<SimulationDescriptorT>) -> bool {
        ! simulation_descriptor.paused
    } 
}
