use std::sync::{Arc, Mutex};

use bevy::app::FixedPostUpdate;
use bevy::window::PrimaryWindow;
//T: Comment for errors
// use bevy::render::RenderPlugin;
// use bevy::{prelude::*, window::WindowResizeConstraints, DefaultPlugins};
use bevy::window::WindowResizeConstraints;
use bevy_egui::EguiPlugin;
// use bevy_prototype_lyon::prelude::ShapePlugin;

use crate::engine::simulation::Simulation;

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::window::WindowPlugin;
use bevy::winit::WinitPlugin;
use bevy::a11y::AccessibilityPlugin;
use bevy::render::RenderPlugin;
use bevy::prelude::PluginGroup;
use bevy::utils::default;

use bevy::prelude::Update;
use bevy::prelude::Startup;

//T: added this for ClearColor
use bevy::prelude::ClearColor;

//T: added this for Insertion of resource Time
use bevy::time::Time;
use bevy::time::Fixed;

use bevy::ecs::system::Resource;

use bevy::prelude::IntoSystemConfigs;
use super::systems::renderer_system::renderer_system;

//T: resolve errors about Color not found in main
// Export Color directly from here with 'pub'
pub use bevy::render::color::Color as Color;

//T: TEMP
// use crate::engine::Query;
// use crate::engine::With;
// use bevy::window::Window;
// use bevy::prelude::Commands;
// use bevy::prelude::Startup;
// use bevy::ecs::system::ResMut;
// use bevy::prelude::Transform;
// use bevy::prelude::Vec2;
// use bevy::render::camera::ScalingMode;
// use bevy::render::camera::OrthographicProjection;
// use bevy::prelude::Camera2dBundle;

//T: This elements are removed from the krabmaga framework
//use crate::engine::{schedule::Schedule, state::State};

//T: Comments for errors
// use crate::visualization::{
//     asset_handle_factory::AssetHandleFactory,
//     simulation_descriptor::SimulationDescriptor,
//     systems::{
//         camera_system::camera_system, init_system::init_system, renderer_system::renderer_system,
//         simulation_system::simulation_system, ui_system::ui_system,
//     },
//     visualization_state::VisualizationState,
//     wrappers::{ActiveSchedule, ActiveState, Initializer},
// };

use crate::visualization::simulation_descriptor::SimulationDescriptor;

use super::systems::camera_system::camera_system;
use super::systems::ui_system::ui_system;
use super::systems::init_system::init_system;

// The application main struct, used to build and start the event loop. Offers several methods in a builder-pattern style
// to allow for basic customization, such as background color, asset path and custom systems. Right now the framework
// supports the automatic visualization of a single type of agents, for ease of implementation.
//
// REQUIREMENTS:
// 1) In the root of the project, a folder called `assets` must be created. The emoji icons used will
//     have to be copied there. In future, this limitation will be removed.
pub struct Visualization {
    width: f32,
    height: f32,
    sim_width: f32,
    sim_height: f32,
    window_name: &'static str,
    background_color: Color,
}

impl Visualization {
    // Specify width and height of the window where the visualization will appear. Defaults to 500x300.
    pub fn with_window_dimensions(mut self, width: f32, height: f32) -> Visualization {
        self.width = width;
        self.height = height;
        self
    }

    // Specify width and height of the simulation. This should not be smaller than the window dimension,
    // or else the simulation won't be fully visible. Defaults to 500.300
    pub fn with_simulation_dimensions(mut self, width: f32, height: f32) -> Visualization {
        self.sim_width = width;
        self.sim_height = height;
        self
    }

    // Specify the name of the window. Defaults to the project name defined in the cargo manifest.
    pub fn with_name(mut self, name: &'static str) -> Visualization {
        self.window_name = name;
        self
    }

    // Specify the background color of the window. Defaults to black.
    pub fn with_background_color(mut self, color: Color) -> Visualization {
        self.background_color = color;
        self
    }

    //T: Commented because i think that we don't need it anymore
    // Create the application and start it immediately. Requires a startup callback defined as a struct
    // that implements [OnStateInit], along with the state and the schedule, which you manually create.
    // pub fn start<I: VisualizationState<S> + 'static + bevy::prelude::Resource + Clone, S: State>(
    //     self,
    //     init_call: I,
    //     state: S,
    // ) {
    //     let mut app_builder = self.setup(init_call, state);
    //     app_builder.run()
    // }

    //T: Probably we don't need anymore this method, for now it produces only error
    pub fn start(self)
    {

    }

    //T: Commented for a great numbers of errors
    // Sets up the application, exposing the [AppBuilder]. Useful if you want to directly interface Bevy
    // and add plugins, resources or systems yourself.
    // pub fn setup<I: VisualizationState<S> + Clone + 'static + bevy::prelude::Resource, S: State>(
    //     &self,
    //     init_call: I,
    //     mut state: S,
    // ) -> App {
    //     //Minimum constraints taking into account a 300 x 300 simulation window + a 300 width UI panel
    //     let mut window_constraints = WindowResizeConstraints::default();
    //     window_constraints.min_width = 600.;
    //     window_constraints.min_height = 300.;

    //     let mut app = App::new();
    //     let mut schedule = Schedule::new();
    //     state.init(&mut schedule);
    //     let cloned_init_call = init_call.clone();

        // app.add_plugins(DefaultPlugins.set(RenderPlugin {
        //     // Resolves false positive error spam in console for AMD GPUs, but it breaks WebGL: https://github.com/bevyengine/bevy/issues/9975
        //     // render_creation: RenderCreation::Automatic(WgpuSettings {
        //     //     backends: Some(Backends::VULKAN),
        //     //     ..default()
        //     // }),
        //     ..default()
        // }))
        // .add_plugins(EguiPlugin);

    //     // Required for network visualization
    //     app.add_plugins(ShapePlugin);

    //     app.insert_resource(SimulationDescriptor {
    //         title: self
    //             .window_name
    //             .parse()
    //             .expect("Error: can't parse window name"),
    //         width: self.sim_width,
    //         height: self.sim_height,
    //         center_x: (self.width * 0.5) - (self.width - self.sim_width as f32) / 2.,
    //         center_y: (self.height * 0.5) - (self.height - self.sim_height as f32) / 2.,
    //         paused: true,
    //         ui_width: 300.,
    //     })
    //     .insert_resource(ClearColor(self.background_color))
    //     .insert_resource(AssetHandleFactory::new())
    //     .insert_resource(init_call)
    //     .insert_resource(ActiveState(Arc::new(Mutex::new(state))))
    //     .insert_resource(ActiveSchedule(Arc::new(Mutex::new(schedule))))
    //     .insert_resource(Initializer(cloned_init_call, Default::default()))
    //     .add_systems(FixedPreUpdate, simulation_system::<S>)
    //     .add_systems(Update, ui_system::<I, S>)
    //     .add_systems(FixedPostUpdate, renderer_system::<I, S>)
    //     .insert_resource(Time::<Fixed>::default())
    //     .add_systems(Startup, init_system::<I, S>)
    //     .add_plugins(FrameTimeDiagnosticsPlugin::default())
    //     .add_systems(Update, camera_system);

    //     app
    // }

    //T: rewriting this functions
    //T: TODO try to create a way to pass the g_initializer throug a different function
    //T: and execute that after init_system
    pub fn setup<Params>(
        &self, 
        simulation: &mut Simulation,
        g_initializer: impl IntoSystemConfigs<Params>,
    ) {
        let mut app = &mut simulation.app;

        //T: TODO Add plugins here
        // app.add_plugins(AccessibilityPlugin {});
        // app.add_plugins(WindowPlugin {..default()});
        // app.add_plugins(WinitPlugin {..default()});
        // app.add_plugins(RenderPlugin {..default()});

        //Minimum constraints taking into account a 300 x 300 simulation window + a 300 width UI panel
        let mut window_constraints = WindowResizeConstraints::default();
        window_constraints.min_width = 600.;
        window_constraints.min_height = 300.;

        app.add_plugins(EguiPlugin);
        app.add_plugins(FrameTimeDiagnosticsPlugin::default());

        //T: added at startup this system
        app.add_systems(Startup, 
            (init_system, g_initializer.after(init_system)));

        app.add_systems(Update, ui_system);
        app.add_systems(Update, camera_system);

        //T: added temporary
        app.add_systems(Update, renderer_system);

        app.insert_resource(Time::<Fixed>::default());
        app.insert_resource(ClearColor(self.background_color));
        app.insert_resource(SimulationDescriptor {
            title: self
                .window_name
                .parse()
                .expect("Error: can't parse window name"),
            width: self.sim_width,
            height: self.sim_height,
            center_x: (self.width * 0.5) - (self.width - self.sim_width as f32) / 2.,
            center_y: (self.height * 0.5) - (self.height - self.sim_height as f32) / 2.,
            paused: true,
            ui_width: 300.,
        });
    }
}

impl Default for Visualization {
    fn default() -> Self {
        Visualization {
            width: 600.,
            height: 300.,
            sim_width: 300.,
            sim_height: 300.,
            window_name: env!("CARGO_PKG_NAME"),
            background_color: Color::rgb(1., 1., 1.),
        }
    }
}
