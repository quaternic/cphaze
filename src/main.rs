#![feature(new_range_api)]
#![feature(btree_cursors)]

use bevy::{
    prelude::*,
    render,
};
use point_cloud::PointCloudPipelinePlugin;

mod plane;
mod incremental;
mod ui;
mod orbit_cam;
mod lines;
mod func_xy;
mod point_cloud;

#[derive(Component)]
pub struct PanningCamera;

fn main() {
    use bevy::core::TaskPoolThreadAssignmentPolicy;

    let wgpu = render::settings::WgpuSettings::default();
    let render = render::RenderPlugin {
        render_creation: render::settings::RenderCreation::Automatic(wgpu),
        synchronous_pipeline_compilation: true,
    };
    let taskpool = TaskPoolPlugin {
        task_pool_options: TaskPoolOptions {
            min_total_threads: 3,
            max_total_threads: 3,
            compute: TaskPoolThreadAssignmentPolicy {
                min_threads: 1,
                max_threads: 1,
                percent: 1.0,
            },
            io: TaskPoolThreadAssignmentPolicy {
                min_threads: 1,
                max_threads: 1,
                percent: 1.0,
            },
            async_compute: TaskPoolThreadAssignmentPolicy {
                min_threads: 1,
                max_threads: 1,
                percent: 1.0,
            },
        }
    };
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(taskpool)
                .set(render),
            MaterialPlugin::<func_xy::ParticleMaterial>::default(),
            incremental::GpuReadbackPlugin,
            func_xy::PluginXY,
            plane::plugin,
            bevy_egui::EguiPlugin,
            PointCloudPipelinePlugin,
        ))
        .register_type::<orbit_cam::OrbitState>()
        .add_systems(Startup, orbit_cam::spawn_camera)
        .add_systems(Update, orbit_cam::orbit_camera)

        .add_systems(Startup, lines::setup)
        .add_systems(Update, ui::ui_system)
        .run();
}

#[hot_lib_reloader::hot_module(dylib = "lib", file_watch_debounce = 50)]
mod hot {
    hot_functions_from_file!("lib/src/lib.rs");
}