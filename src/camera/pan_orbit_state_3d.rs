use crate::{
    app::{app_settings::AppSettings, app_state::AppState},
    camera::main_camera::{ApplyCameraState, CameraMode, Invalidate},
    rendering::tiled_image::TiledImage,
};
use bevy::prelude::{EulerRot, Projection, Quat, Query, Resource, Transform, Vec2, Vec3};
use bitflags::Flags;
use std::f32::consts::{FRAC_PI_2, PI, TAU};

#[derive(Resource, Clone)]
pub(crate) struct PanOrbitState3d {
    pub(crate) center: Vec3,
    pub(crate) radius: f32,
    pub(crate) pitch: f32,
    pub(crate) yaw: f32,
    pub(crate) is_added: bool,
}

impl PanOrbitState3d {
    pub(crate) fn new(center: Vec3, radius: f32, pitch: f32, yaw: f32, is_added: bool) -> Self {
        Self {
            center,
            radius,
            pitch,
            yaw,
            is_added,
        }
    }
}

impl Default for PanOrbitState3d {
    fn default() -> Self {
        PanOrbitState3d {
            center: Vec3::ZERO,
            radius: 1.0,
            pitch: 0.0,
            yaw: 0.0,
            is_added: true,
        }
    }
}

impl ApplyCameraState for PanOrbitState3d {
    fn get_initial_state(&self, _: &Transform, _: &Projection) -> Self {
        self.clone()
    }

    fn apply(
        &mut self,
        mode: CameraMode,
        initial_state: &Self,
        _: Vec2,
        _: Vec2,
        delta_zoom: f32,
        delta_move: Vec3,
        app_settings: &AppSettings,
        _: &mut AppState,
        _: Query<&TiledImage>,
        transform: &mut Transform,
        _: &mut Projection,
        invalidate: &mut Invalidate,
    ) {
        // Taken from https://bevy-cheatbook.github.io/cookbook/pan-orbit-camera.html
        let mut any = false;

        if mode.intersects(CameraMode::Orbit) && delta_move != Vec3::ZERO {
            any = true;

            // If we are upside down, reverse the X orbiting
            let delta_move_x =
                if initial_state.pitch < -FRAC_PI_2 || initial_state.pitch > FRAC_PI_2 {
                    delta_move.x
                } else {
                    -delta_move.x
                };

            self.yaw = initial_state.yaw
                + delta_move_x * app_settings.pan_orbit_settings.orbit_sensitivity;
            self.pitch = initial_state.pitch
                + delta_move.y * app_settings.pan_orbit_settings.orbit_sensitivity;
            // wrap around, to stay between +- 180 degrees
            if self.yaw > PI {
                self.yaw -= TAU; // 2 * PI
            }
            if self.yaw < -PI {
                self.yaw += TAU; // 2 * PI
            }
            if self.pitch > PI {
                self.pitch -= TAU; // 2 * PI
            }
            if self.pitch < -PI {
                self.pitch += TAU; // 2 * PI
            }
        }

        if mode.intersects(CameraMode::Zoom) && delta_zoom != 1.0 {
            any = true;
            // in order for zoom to feel intuitive,
            // everything needs to be exponential
            // (done via multiplication)
            // not linear
            // (done via addition)

            // so we compute the exponential of our
            // accumulated value and multiply by that
            self.radius = initial_state.radius * delta_zoom;
        }

        // To PAN, we can get the UP and RIGHT direction
        // vectors from the camera's transform, and use
        // them to move the center point. Multiply by the
        // radius to make the pan adapt to the current zoom.
        if mode.intersects(CameraMode::Pan) && delta_move != Vec3::ZERO {
            any = true;
            self.center = initial_state.center
                + transform.right()
                    * (-delta_move.x)
                    * app_settings.pan_orbit_settings.pan_sensitivity
                    * self.radius
                + transform.up()
                    * delta_move.y
                    * app_settings.pan_orbit_settings.pan_sensitivity
                    * self.radius;
        }

        // Finally, compute the new camera transform.
        // (if we changed anything, or if the pan-orbit
        // controller was just added and thus we are running
        // for the first time and need to initialize)
        if any || self.is_added {
            // YXZ Euler Rotation performs yaw/pitch/roll.
            transform.rotation = Quat::from_euler(EulerRot::YXZ, self.yaw, self.pitch, 0.0);
            // To position the camera, get the backward direction vector
            // and place the camera at the desired radius from the center.
            transform.translation = self.center + transform.back() * self.radius;

            self.is_added = false;
        }

        invalidate.clear();
    }
}
