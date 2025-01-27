use bevy::{core_pipeline::tonemapping::Tonemapping, input::mouse::MouseScrollUnit, math::vec2, prelude::*};
use bevy_egui::egui::{self, Widget};

#[derive(Component, Reflect)]
pub struct OrbitState {
    pub target: Vec3,
    pub radius: f32,
    pub rotation: Quat,
    pub auto_rotate_z: bool,
}

impl Default for OrbitState {
    fn default() -> Self {
        Self {
            target: Vec3::Z,
            radius: 6.0,
            rotation: Quat::from_array([0.56,-0.1,-0.15,0.8]).normalize(),
            auto_rotate_z: false,
        }
    }
}

impl Widget for &mut OrbitState {
    fn ui(self, ui: &mut bevy_egui::egui::Ui) -> bevy_egui::egui::Response {
        ui.collapsing("Camera", |ui| {
            ui.add(egui::Slider::new(&mut self.target.z, 0.0..=10.0));
            ui.checkbox(&mut self.auto_rotate_z, "rotate Z-axis")
        }).header_response
    }
}

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Camera {
            hdr: true,
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        Tonemapping::Reinhard,
        OrbitState::default(),
    ));
}

pub fn orbit_camera(
    _kbd: Res<ButtonInput<KeyCode>>,
    click: Res<ButtonInput<MouseButton>>,
    mut evr_motion: EventReader<bevy::input::mouse::MouseMotion>,
    mut evr_scroll: EventReader<bevy::input::mouse::MouseWheel>,
    mut q_camera: Query<(
        &mut OrbitState,
        &mut Transform,
    )>,
    egui_contexts: Query<&bevy_egui::EguiContext>,
    time: Res<Time<Real>>,
) {
    let get_mouse = !egui_contexts.iter().any(|ctx| ctx.get().wants_pointer_input());

    let mut total_motion = Vec2::ZERO;
    let mut scroll = Vec2::ZERO;
    if get_mouse {
        if click.pressed(MouseButton::Left) {
            for ev in evr_motion.read() {
                total_motion += ev.delta;
            }
        } else {
            evr_motion.clear();
        }
        
        for ev in evr_scroll.read() {
            scroll -= vec2(ev.x, ev.y) * match ev.unit {
                MouseScrollUnit::Line => 1.0 / 8.0,
                MouseScrollUnit::Pixel => 1.0 / 64.0,
            };
        }
    }

    for (mut state, mut transform) in &mut q_camera {
        let x = total_motion.x / 256.0;
        let y = total_motion.y / 256.0;
        state.rotation *= Quat::from_array([-y,-x,0.0,1.0]);
        state.rotation = state.rotation.normalize();

        if state.auto_rotate_z {
            state.rotation = Quat::from_rotation_z(-0.3 * time.delta_secs()) * state.rotation;
        }

        state.radius *= scroll.y.exp2();

        transform.rotation = state.rotation;
        transform.translation = state.target + transform.back() * state.radius;

    }
}