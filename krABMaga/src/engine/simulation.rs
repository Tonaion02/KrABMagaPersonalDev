use core::num;
use std::thread;
use std::time::Duration;

use bevy::core::TaskPoolThreadAssignmentPolicy;
use bevy::log::LogPlugin;
use bevy::prelude::*;

use crate::engine::fields::field_2d::{update_field, Field2D};
use crate::engine::resources::engine_configuration::EngineConfiguration;
use crate::engine::rng::RNG;
use crate::engine::systems::double_buffer_sync::double_buffer_sync;
use crate::engine::systems::engine_config_update::engine_config_update;

//T: for testing of plugins
use bevy::time::TimePlugin;
use bevy::diagnostic::DiagnosticsPlugin;
use bevy::input::InputPlugin;
use bevy::winit::WinitPlugin;
use bevy::render::RenderPlugin;
use bevy::render::pipelined_rendering::PipelinedRenderingPlugin;
use bevy::core_pipeline::CorePipelinePlugin;
use bevy::sprite::SpritePlugin;
use bevy::asset::AssetPlugin;
use bevy::a11y::AccessibilityPlugin;




#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum SimulationSet {
    BeforeStep,
    Step,
    AfterStep,
}

pub struct Simulation {
    pub(crate) app: App,
    steps: Option<u32>,

    //T: added to add only at the end 
    num_threads: usize,
}

impl Simulation {
    pub fn build() -> Self {

        let mut app = App::new();
        //T: removed for now for the problems of the plugins
        //T: TODO reactivate it......
        // #[cfg(feature = "trace_tracy")]
        // app.add_plugins(LogPlugin::default());



        // T: trying to make working with less plugins possible
        app.add_plugins(LogPlugin::default());
        app.add_plugins(TaskPoolPlugin::default());
        app.add_plugins(TypeRegistrationPlugin);
        app.add_plugins(FrameCountPlugin);
        app.add_plugins(TimePlugin);
        app.add_plugins(TransformPlugin);
        app.add_plugins(HierarchyPlugin);
        app.add_plugins(DiagnosticsPlugin);
        app.add_plugins(InputPlugin);
        app.add_plugins(WindowPlugin::default());
        app.add_plugins(AccessibilityPlugin);
    
        app.add_plugins(AssetPlugin::default());
        app.add_plugins(WinitPlugin::default());
            
        app.add_plugins(RenderPlugin::default());
        app.add_plugins(ImagePlugin::default());
        //app.add_plugins(PipelinedRenderingPlugin {..default()});
        app.add_plugins(CorePipelinePlugin::default());
        app.add_plugins(SpritePlugin);
            
    

        app.configure_sets(
            Update,
            (
                SimulationSet::BeforeStep,
                SimulationSet::Step,
                SimulationSet::AfterStep,
            )
                .chain(),
        );

        //T: In case where is not defined feature visualization
        #[cfg(not(feature = "visualization"))]
        app.add_systems(
                Update,
                (engine_config_update,).in_set(SimulationSet::BeforeStep),
        );
        //T: In case where is not defined feature visualization

        //T: In case where is defined feature visualization 
        #[cfg(feature = "visualization")]
        app.add_systems(
             FixedPreUpdate,
             (engine_config_update,).in_set(SimulationSet::BeforeStep),
        );
        //T: In case where is defined feature visualization

        Self { app, steps: None, num_threads: 0 }
    }

    pub fn with_num_threads(mut self, num_threads: usize) -> Self {
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
        self.app.add_systems(FixedPreUpdate, (step_handler,).in_set(SimulationSet::Step));
        
        self
    }

    // TODO figure out a way to automatically register double buffers
    pub fn register_double_buffer<T: Component + Copy + Send>(mut self) -> Self {
        self.app.add_systems(
            Update,
            (double_buffer_sync::<T>,).in_set(SimulationSet::BeforeStep),
        );
        
        self
    }

    pub fn with_steps(mut self, steps: u32) -> Self {
        self.steps = Some(steps);

        self
    }

    // TODO specify this is required (SimulationBuilder with validation, which generates a Simulation on build()?)
    pub fn with_engine_configuration(mut self, config: EngineConfiguration) -> Self {
        self.app.insert_resource(config);

        self
    }

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

    // pub fn add_plugins(&mut self) {
    //     self.app.add_plugins(DefaultPlugins.set(TaskPoolPlugin {
    //         task_pool_options: TaskPoolOptions {
    //             // Assign all threads to compute
    //             compute: TaskPoolThreadAssignmentPolicy {
    //                 // set the minimum # of compute threads
    //                 // to the total number of available threads
    //                 min_threads: 4,
    //                 max_threads: 4, // unlimited max threads
    //                 percent: 1.0,             // this value is irrelevant in this case
    //             },
    //             ..default()
    //         },
    //     }));
    // }
}
