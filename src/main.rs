#![feature(new_range_api)]
#![feature(btree_cursors)]
#![feature(panic_update_hook)]
#![feature(panic_backtrace_config)]
#![feature(backtrace_frames)]

use std::{backtrace::Backtrace, panic::BacktraceStyle};

use bevy::{
    app::PanicHandlerPlugin, prelude::*, render
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
                .set(render)
                .disable::<PanicHandlerPlugin>(),
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
        .add_systems(Startup, || {
            PANIC_INFO.set(Some((String::new(), None)));
            std::panic::set_hook(Box::new(|info| {
                PANIC_INFO.with(|stash| {
                    let old = stash.take();
                    if old.is_none() {
                        let style = std::panic::get_backtrace_style();
                        let bt = if let Some(BacktraceStyle::Full | BacktraceStyle::Short) = style {
                            Some(std::backtrace::Backtrace::force_capture())
                        } else {
                            None
                        };
                        let s = format!("{info}",
                        );
                        stash.set(Some((s,bt)));
                    } else {
                        stash.set(old);
                    }
                });
            }));
        })
        .add_systems(Update, ui::ui_system)
        .run();
}

use std::cell::Cell;
thread_local! {
    static PANIC_INFO: Cell<Option<(String, Option<Backtrace>)>> = const { Cell::new(None) };
}

#[hot_lib_reloader::hot_module(dylib = "lib", file_watch_debounce = 50)]
mod hot {
    hot_functions_from_file!("lib/src/lib.rs");
}
