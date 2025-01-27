

use bevy::prelude::*;
use bevy_egui::{egui::{self, Ui},EguiContexts};

use crate::{func_xy::{FuncXY, InputPoints, ParticleMaterial, RegionUpdate}, orbit_cam::OrbitState};


pub fn ui_system(
    mut contexts: EguiContexts,
    mut commands: Commands,
    mut points: ResMut<InputPoints>,
    mut x_bits: Local<(i32, i32)>,
    mut y_bits: Local<(i32, i32)>,
    mut q_func: Query<(&FuncXY,&MeshMaterial3d<ParticleMaterial>, &mut Visibility)>,
    mut materials: ResMut<Assets<ParticleMaterial>>,
    mut cams: Query<&mut OrbitState>,
) {
    let points = &mut *points;
    egui::Window::new("Controls").show(contexts.ctx_mut(), |ui| {

        let button = |ui: &mut Ui, set, unset, chars: &str| {
            let mut state = (set as i32) - (unset as i32);
            let i = (1 + state) as usize;
            let r = ui.small_button(&chars[i..i+1]);
            if r.clicked_by(egui::PointerButton::Primary) {
                if state == 0 {
                    state = 1;
                } else {
                    state = -state;
                }
            } else if r.clicked_by(egui::PointerButton::Secondary) {
                if state == 0 {
                    state = -1;
                } else {
                    state = 0;
                }
            }
            ((state > 0) as i32, (state < 0) as i32)
        };

        ui.horizontal(|ui| {
            ui.label("x: ");

            let mut os = x_bits.1;
            let mut ou = x_bits.0;
            let mut ns = 0;
            let mut nu = 0;
            ns <<= 1; nu <<= 1;
            let (mut s, mut u) = button(ui, os < 0, ou < 0, "- +");
            ns += s;  nu += u; 
            os <<= 1; ou <<= 1;

            for _ in 1..32 {
                ns <<= 1; nu <<= 1;
                if s | u != 0 {
                    (s,u) = button(ui, os < 0, ou < 0, "0 1");
                    ns += s; nu += u;
                }
                os <<= 1; ou <<= 1;
            }
            *x_bits = (nu,ns);
        });
        ui.horizontal(|ui| {
            ui.label("y: ");

            let mut os = y_bits.1;
            let mut ou = y_bits.0;
            let mut ns = 0;
            let mut nu = 0;
            ns <<= 1; nu <<= 1;
            let (mut s, mut u) = button(ui, os < 0, ou < 0, "- +");
            ns += s;  nu += u; 
            os <<= 1; ou <<= 1;

            for _ in 1..32 {
                ns <<= 1; nu <<= 1;
                if s | u != 0 {
                    (s,u) = button(ui, os < 0, ou < 0, "0 1");
                    ns += s; nu += u;
                }
                os <<= 1; ou <<= 1;
            }
            *y_bits = (nu,ns);
        });

        let min = x_bits.1 ^ i32::MIN;
        let max = !x_bits.0 ^ i32::MIN;

        commands.trigger(RegionUpdate::StartX(min));
        commands.trigger(RegionUpdate::EndX(max));

        let min = y_bits.1 ^ i32::MIN;
        let max = !y_bits.0 ^ i32::MIN;

        commands.trigger(RegionUpdate::StartY(min));
        commands.trigger(RegionUpdate::EndY(max));

        for (s, x) in [
            ("x0: ", points.x_range.start),
            ("x1: ", points.x_range.end),
            ("y0: ", points.y_range.start),
            ("y1: ", points.y_range.end),
        ] {
            ui.monospace(format!("{s}{}", crate::hot::int_fmt(x)));
        }

        ui.group(|ui| {
            ui.add(&mut *points);
        });

        if points.max_len != points.x_vec.len() as u32 {
            commands.trigger(RegionUpdate::SetLen(points.max_len));
        }

        ui.collapsing("Functions", |ui| {
            for (_f, mat, mut visibility) in q_func.iter_mut() {
                ui.horizontal(|ui| {

                    let mut show = *visibility != Visibility::Hidden;
                    ui.checkbox(&mut show, "");
                    if show {
                        *visibility = Visibility::Inherited;
                    } else {
                        *visibility = Visibility::Hidden;
                    }

                    ui.add(materials.get_mut(mat).unwrap());
                });
            }

        });
        
        for mut cam in cams.iter_mut() {
            ui.add(&mut *cam);
        }
    });
}
