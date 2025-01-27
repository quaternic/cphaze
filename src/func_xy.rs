use core::range::RangeInclusive;
use std::{collections::BTreeSet, mem};

use bevy::render::extract_resource::ExtractResource;
use bevy::{prelude::*, render::storage::ShaderStorageBuffer};
use bevy_egui::egui::{self, Slider, Widget};
use rand::{distributions::Uniform, prelude::Distribution, rngs::StdRng, Rng, SeedableRng};

use bevy::render::render_resource::{AsBindGroup, BufferUsages};
use bevy::render::render_asset::RenderAssetUsages;

use crate::point_cloud::PointCloudEntity;
use crate::plane::PlaneMaterial;

pub struct PluginXY;

impl Plugin for PluginXY {
    fn build(&self, app: &mut App) {
        app.add_observer(change_region)
            .add_systems(Startup, setup)
            .add_systems(Update, track_time);
    }
}

/// A function to be evaluated at points of R^2
#[derive(Component)]
pub struct FuncXY {
    pub id: u32,
    pub zs: Handle<ShaderStorageBuffer>,
}


/// Event that modifies the input region
#[derive(Event)]
pub enum RegionUpdate {
    StartX(i32),
    StartY(i32),
    EndX(i32),
    EndY(i32),
    SetLen(u32),
}


/// The points sampled from the input region
#[derive(Resource)]
pub struct InputPoints {
    pub x_vec: Vec<i32>,
    pub y_vec: Vec<i32>,

    // indices of points that have been relocated and need updating
    pub modified: BTreeSet<u32>,

    pub x_range: RangeInclusive<i32>,
    pub y_range: RangeInclusive<i32>,

    pub xs: Handle<ShaderStorageBuffer>,
    pub ys: Handle<ShaderStorageBuffer>,

    pub max_len: u32,

    pub refresh_random: bool,
    pub refresh_rate: u32,
}

impl InputPoints {
    const MAX_LEN: u32 = 1 << 20;
}


impl Widget for &mut InputPoints {
    fn ui(self, ui: &mut bevy_egui::egui::Ui) -> egui::Response {
        ui.add(egui::Label::new(format!("update queue: {}", self.modified.len())));
        ui.add(egui::Label::new("updates per frame:"));
        ui.add(egui::Slider::new(&mut self.refresh_rate, 0..=(1 << 17)).logarithmic(true));
        ui.add(egui::Checkbox::new(&mut self.refresh_random, "refresh random points"));
        ui.add(egui::Slider::new(&mut self.max_len, (1 << 10)..=InputPoints::MAX_LEN).logarithmic(true));
        ui.response()
    }
}

pub fn setup(
    mut commands: Commands,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {

    let mut xs = ShaderStorageBuffer::with_size(InputPoints::MAX_LEN as usize * size_of::<i32>(), RenderAssetUsages::RENDER_WORLD);
    xs.set_data(vec![0; InputPoints::MAX_LEN as usize]);
    xs.buffer_description.usage |= BufferUsages::VERTEX;
    let xs = buffers.add(xs);
    let mut ys = ShaderStorageBuffer::with_size(InputPoints::MAX_LEN as usize * size_of::<i32>(), RenderAssetUsages::RENDER_WORLD);
    ys.set_data(vec![0; InputPoints::MAX_LEN as usize]);
    ys.buffer_description.usage |= BufferUsages::VERTEX;
    let ys = buffers.add(ys);

    commands.insert_resource(InputPoints {
        xs: xs.clone(),
        ys: ys.clone(),
        modified: default(),
        x_vec: vec![],
        y_vec: vec![],

        x_range: (i32::MIN..=i32::MAX).into(),
        y_range: (i32::MIN..=i32::MAX).into(), 
        max_len: InputPoints::MAX_LEN,

        refresh_random: true,
        refresh_rate: 100,
    });
    
    commands.add_observer(spawn_points);

    commands.trigger(Spawn(0, LinearRgba::new(1.0,0.0,0.0,0.0)));
    commands.trigger(Spawn(1, LinearRgba::new(0.0,1.0,0.0,0.0)));
    commands.trigger(Spawn(2, LinearRgba::new(0.0,0.0,1.0,0.0)));
}
pub fn track_time(
    t: Res<Time<Virtual>>,
    input: Res<InputPoints>,
    mut materials: ResMut<Assets<ParticleMaterial>>,
    mut materials2: ResMut<Assets<PlaneMaterial>>,

    mut q_points: Query<&mut PointCloudEntity>,
) {
    let bounds = [
        input.x_range.start,
        input.y_range.start,
        input.x_range.end,
        input.y_range.end,
    ].into();
    for (_, mat) in materials.iter_mut() {
        mat.xy_bounds = bounds;
        mat.time = mat.time.wrapping_add((t.delta().as_nanos()) as i32);
    }
    for (_, mat) in materials2.iter_mut() {
        mat.xy_bounds = bounds;
        mat.time = mat.time.wrapping_add((t.delta().as_nanos()) as i32);
    }

    for mut points in q_points.iter_mut() {
        points.init = input.max_len;
    }
}

#[derive(Event)]
pub struct Spawn(
    pub u32,
    pub LinearRgba,
);


pub fn spawn_points(
    trigger: Trigger<Spawn>,
    mut commands: Commands,
    inputs: Res<InputPoints>,
    mut materials: ResMut<Assets<ParticleMaterial>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    let mut zs = ShaderStorageBuffer::with_size(inputs.max_len as usize * mem::size_of::<f32>(), RenderAssetUsages::RENDER_WORLD);
    zs.set_data(vec![0.0; inputs.max_len as usize]);
    zs.buffer_description.usage |= BufferUsages::VERTEX;
    let zs = buffers.add(zs);

    let mat = materials.add(ParticleMaterial {
        time: 0,
        color: trigger.1,
        xy_bounds: [
            inputs.x_range.start,
            inputs.y_range.start,
            inputs.x_range.end,
            inputs.y_range.end,
        ].into(),
        z_scale: 1.0,
    });

    commands.spawn((
        crate::point_cloud::PointCloudEntity {
            xs: inputs.xs.clone(),
            ys: inputs.ys.clone(),
            zs: zs.clone(),
            init: inputs.max_len, // FIXME
        },
        MeshMaterial3d(mat),
        FuncXY {
            id: trigger.0,
            zs,
        },
        Visibility::default(),
        Transform::default(),
    ));
}

// This is the struct that will be passed to your shader
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default, ExtractResource, Resource)]
pub struct ParticleMaterial {
    #[uniform(0)]
    time: i32,
    #[uniform(1)]
    color: LinearRgba,
    #[uniform(2)]
    xy_bounds: IVec4,
    #[uniform(3)]
    z_scale: f32,
}

impl Widget for &mut ParticleMaterial {
    fn ui(self, ui: &mut bevy_egui::egui::Ui) -> bevy_egui::egui::Response {
        let mut color = self.color.to_f32_array_no_alpha();
        let response = ui.color_edit_button_rgb(&mut color);
        self.color = LinearRgba::from_f32_array_no_alpha(color);
        self.color.alpha = 0.0;

        const M: f32 = (1 << 20) as f32;
        ui.add(Slider::new(&mut self.z_scale, 1.0 / M ..= M).logarithmic(true));
        response
    }
}


impl Material for ParticleMaterial {}


fn range_uniform(r: RangeInclusive<i32>) -> Uniform<i32> {
    Uniform::new_inclusive(r.start, r.end)
}

fn change_region(
    trigger: Trigger<RegionUpdate>,
    mut points: ResMut<InputPoints>,
) {
    let points: &mut InputPoints = &mut points;
    let (new, old, other, vec) = match *trigger.event() {
        RegionUpdate::StartX(n) => (n, &mut points.x_range.start, points.x_range.end, &mut points.x_vec),
        RegionUpdate::StartY(n) => (n, &mut points.y_range.start, points.y_range.end, &mut points.y_vec),
        RegionUpdate::EndX(n) =>   (n, &mut points.x_range.end, points.x_range.start, &mut points.x_vec),
        RegionUpdate::EndY(n) =>   (n, &mut points.y_range.end, points.y_range.start, &mut points.y_vec),

        RegionUpdate::SetLen(new) => {
            let new = new.min(points.max_len);
            let old = points.x_vec.len() as u32;
            if new > old {
                points.modified.extend(old..new);
                let ref mut rng = StdRng::from_entropy();
                let x_distr = range_uniform(points.x_range);
                let y_distr = range_uniform(points.y_range);
                points.x_vec.extend(rng.sample_iter(x_distr).take((new - old) as usize));
                points.y_vec.extend(rng.sample_iter(y_distr).take((new - old) as usize));
            } else if new < old {
                drop(points.modified.split_off(&new));
                points.x_vec.truncate(new as usize);
                points.y_vec.truncate(new as usize);
            }
            return;
        }
    };
    let old = mem::replace(old, new);

    let old_len = old.abs_diff(other);
    let new_len = new.abs_diff(other);
    let mut rng = StdRng::from_entropy();

    match new_len.cmp(&old_len) {
        std::cmp::Ordering::Less => {
            let lb = new.min(other);
            let ub = new.max(other);
            let new_distr = Uniform::new_inclusive(lb, ub);
            // shrinking: invalidate values that are out-of-bounds

            
            for (idx, val) in vec.iter_mut().enumerate() {
                if !(lb..=ub).contains(val) {
                    *val = new_distr.sample(&mut rng);
                    points.modified.insert(idx as u32);
                }
            }
        }
        std::cmp::Ordering::Equal => (),
        std::cmp::Ordering::Greater => {
            // growing: pick a random subset of the computed values and give them new values
            let p = (new_len - old_len) as f32 / (new_len as f32 + 1.0);

            let new_range = Uniform::new_inclusive(
                new.min(old + 1),
                new.max(old - 1),
            );


            // Let P("choose the ith item") = p  (for all i)
            // P([1, ..]) = p (choosing the first element)
            // P([0, 1, ..]) = p(1-p) (the second element is the first chosen one)
            // P([0, 0, 1, ..]) = p(1-p)^2 (the third element is the first chosen one)
            // This is the geometric distribution:
            // P("first choice is arr[k]") = p*(1-p)^k
            // P("the first choice is in arr[k..]") = (1-p)^k

            // consider the sequence
            // p(k) = (1-p)^k
            // For all k >= 0,
            //  - 1 >= p(k) > p(k+1) > 0
            //  - P("first choice is arr[k]") = p(k) - p(k+1)

            // Given a uniform X in [0,1)

            // P("first choice is arr[k]")
            //  = P( p(k) >= 1-X > p(k+1) )
            //  = P( (1-p)^k >= 1-X > (1-p)^(k+1) )
            //  = P( k*log(1-p) >= log(1-X) > (k+1)*log(1-p) )
            //  = P( k <= log(1-X)/log(1-p) < k+1 )

            assert!(p > 0.0);
            assert!(p <= 1.0);
            
            const EPS: f32 = 1.0 / (1u64 << 32) as f32;
            let ln1mp = f32::ln_1p(- p);
            let rln1mp = 1.0 / ln1mp;
            let mut idx = 0;
            loop {
                let x: f32 = rng.r#gen::<u32>() as f32 * EPS;
                // saturating cast to u32 first
                idx += (f32::ln_1p(- x) * rln1mp) as u32 as usize;
                //info!("x = {x}, ln(1-x) = {}, ln(1-x)*rln1mp = {}, rln1mp = {rln1mp}", f32::ln_1p(- x),  f32::ln_1p(- x) * rln1mp);

                if let Some(val) = vec.get_mut(idx) {
                    *val = new_range.sample(&mut rng);
                    points.modified.insert(idx as u32);
                    idx += 1;
                } else {
                    break;
                }
            }
        }
    }
}
