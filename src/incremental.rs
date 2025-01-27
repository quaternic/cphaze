
use std::ops::Bound;

use bevy::{
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_asset::RenderAssets,
        render_graph::{self, RenderGraph, RenderLabel},
        render_resource::{
            binding_types::storage_buffer,
            *,
        }, renderer::{RenderContext, RenderDevice, RenderQueue}, storage::{GpuShaderStorageBuffer, ShaderStorageBuffer}, Render, RenderApp, RenderSet
    },
};
use binding_types::uniform_buffer;

use crate::func_xy::{FuncXY, InputPoints, ParticleMaterial};

/// This example uses a shader source file from the assets subdirectory
const SHADER_ASSET_PATH: &str = "shaders/updater.wgsl";

// The length of the buffer sent to the gpu
//const BUFFER_LEN: usize = 1 << 12;

// We need a plugin to organize all the systems and render node required for this example
pub struct GpuReadbackPlugin;

// The small buffers containing the entries to update this frame
#[derive(Resource, ExtractResource, Clone, Default)]
pub struct UpdateBuffer {
    idxs: Handle<ShaderStorageBuffer>,
    updates: Vec<(
        Handle<ShaderStorageBuffer>,
        Handle<ShaderStorageBuffer>,
    )>,
    len: u32,
}
#[derive(AsBindGroup)]
pub struct UpdateUniforms {
    #[uniform(0)]
    len: u32,
    #[storage(1, read_only)]
    idxs: Handle<ShaderStorageBuffer>,
    #[storage(2, read_only)]
    src: Handle<ShaderStorageBuffer>,
    #[storage(3)]
    dst: Handle<ShaderStorageBuffer>,
}


#[derive(Resource)]
struct GpuBufferBindGroup(Vec<BindGroup>);
#[derive(Resource, ExtractResource, Clone)]
struct ComputePipeline {
    layout: BindGroupLayout,
    pipeline: CachedComputePipelineId,
}

/// Label to identify the node in the render graph
#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct ComputeNodeLabel;

/// The node that will execute the compute shader
#[derive(Default)]
struct ComputeNode {

}

impl Plugin for GpuReadbackPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractResourcePlugin::<UpdateBuffer>::default());
        app.add_plugins(ExtractResourcePlugin::<ParticleMaterial>::default());
        app.add_systems(Update, periodic_updates);
        app.init_resource::<UpdateBuffer>();
        
        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(Render,prepare_bind_group.in_set(RenderSet::PrepareBindGroups));
        // Add the compute node as a top level node to the render graph
        // This means it will only execute once per frame
        let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();
        render_graph.add_node(ComputeNodeLabel, ComputeNode::default());
        render_graph.add_node_edge(ComputeNodeLabel, bevy::render::graph::CameraDriverLabel);
    }

    fn finish(&self, app: &mut App) {
        app.sub_app_mut(RenderApp).init_resource::<ComputePipeline>();
    }
}

impl FromWorld for ComputePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let layout = render_device.create_bind_group_layout(
            None,
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    uniform_buffer::<u32>(false),
                    storage_buffer::<Vec<u32>>(false),
                    storage_buffer::<Vec<f32>>(false),
                    storage_buffer::<Vec<f32>>(false),
                ),
            ),
        );
        let shader = world.load_asset(SHADER_ASSET_PATH);
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("GPU readback compute shader".into()),
            layout: vec![layout.clone()],
            push_constant_ranges: vec![],
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: "main".into(),
            zero_initialize_workgroup_memory: true,
        });
        ComputePipeline { layout, pipeline }
    }
}


// FIXME: under some conditions (TBD), the set of points can become non-uniform
fn periodic_updates(
    mut updates: ResMut<UpdateBuffer>,
    mut inputs: ResMut<InputPoints>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    mut q_func: Query<&FuncXY>,
) {
    let inputs = &mut *inputs;
    updates.updates.clear();

    use rand::Rng;
    let mut rng = rand::thread_rng();
    let len = inputs.refresh_rate;

    let mut idxs: Vec<u32> = Vec::with_capacity(len as _);
    let mut idx_map = std::collections::HashMap::new();
    let mut xs: Vec<i32> = Vec::with_capacity(len as _);
    let mut ys: Vec<i32> = Vec::with_capacity(len as _);


    let mut modified = inputs.modified.lower_bound_mut(Bound::Unbounded);

    for _ in 0..len {
        let idx = match modified.remove_next() {
            Some(i) => i,
            None => {
                if !inputs.refresh_random { break }
                let mut i = inputs.x_vec.len() as u32;

                if i < inputs.max_len {
                    // placeholders
                    inputs.x_vec.push(i32::MAX);
                    inputs.y_vec.push(i32::MAX);
                } else {
                    i = rng.r#gen::<u32>() % inputs.max_len;
                }
                inputs.x_vec[i as usize] = rng.gen_range(inputs.x_range.start..=inputs.x_range.end);
                inputs.y_vec[i as usize] = rng.gen_range(inputs.y_range.start..=inputs.y_range.end);
                i
            }
        };
        match idx_map.entry(idx) {
            std::collections::hash_map::Entry::Occupied(occupied_entry) => {
                let slot = *occupied_entry.get();
                assert_eq!(idxs[slot], idx);
                xs[slot] = inputs.x_vec[idx as usize];
                ys[slot] = inputs.y_vec[idx as usize];

            }
            std::collections::hash_map::Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(idxs.len());
                idxs.push(idx);
                xs.push(inputs.x_vec[idx as usize]);
                ys.push(inputs.y_vec[idx as usize]);
            }
        }
    }

    let len = idxs.len() as u32;

    let idxs = buffers.add(ShaderStorageBuffer::from(idxs));
    updates.len = len;
    updates.idxs = idxs;

    let mut r = vec![0.0; xs.len()];
    for FuncXY { id, zs } in &mut q_func {
        crate::hot::test_batched(&xs, &ys, &mut r, *id);
        updates.updates.push((
            buffers.add(ShaderStorageBuffer::from(r.clone())),
            zs.clone()
        ));
    }
    let xs = buffers.add(ShaderStorageBuffer::from(&xs));
    let ys = buffers.add(ShaderStorageBuffer::from(&ys));
    updates.updates.push((xs, inputs.xs.clone()));
    updates.updates.push((ys, inputs.ys.clone()));
}


fn prepare_bind_group(
    mut commands: Commands,
    pipeline: Res<ComputePipeline>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut updates: ResMut<UpdateBuffer>,
    buffers: Res<RenderAssets<GpuShaderStorageBuffer>>,
) {
    let mut bind_groups = vec![];
    let idxs = buffers.get(&updates.idxs).unwrap();
    let mut uniform = UniformBuffer::from(updates.len);
    uniform.write_buffer(&render_device, &render_queue);

    for (src, dst) in &updates.updates {
        let src = buffers.get(src).unwrap();
        let dst = buffers.get(dst).unwrap();
        let bind_group = render_device.create_bind_group(
            None,
            &pipeline.layout,
            &BindGroupEntries::sequential((
                uniform.into_binding(),
                idxs.buffer.as_entire_buffer_binding(),
                src.buffer.as_entire_buffer_binding(),
                dst.buffer.as_entire_buffer_binding(),
            )),
        );
        bind_groups.push(bind_group);
    }
    updates.updates.clear();
    commands.insert_resource(GpuBufferBindGroup(bind_groups));
}




impl render_graph::Node for ComputeNode {
    fn update(&mut self, _world: &mut World) {
        
    }
    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let &UpdateBuffer { len, .. } = world.resource();
        if len == 0 { return Ok(()) }

        let pipeline_cache = world.resource::<PipelineCache>();
        let &ComputePipeline { pipeline, .. } = world.resource();
        let GpuBufferBindGroup(bind_groups) = world.resource();

        if let Some(init_pipeline) = pipeline_cache.get_compute_pipeline(pipeline) {
            let mut pass = render_context
                .command_encoder()
                .begin_compute_pass(&ComputePassDescriptor::default());

            pass.set_pipeline(init_pipeline);

            for bind_group in bind_groups {
                pass.set_bind_group(0, bind_group, &[]);
                pass.dispatch_workgroups(1, 1, 1);
            }
            drop(pass);
        }
        Ok(())
    }
}
