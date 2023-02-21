use glam::*;

pub struct PerspectiveProperties {
    pub fov_radians: f32,
    pub aspect_ratio: f32,
    pub z_near: f32,
    pub z_far: f32,
}

impl PerspectiveProperties {
    pub fn new(width: u32, height: u32, fov_radians: f32, z_near: f32, z_far: f32) -> Self {
        Self {
            fov_radians,
            aspect_ratio: width as f32 / height as f32,
            z_near,
            z_far,
        }
    }
}

pub struct PerspectiveCamera {
    perspective_props: PerspectiveProperties,
    pub camera_position: glam::Vec3,
    pub view_forward: glam::Vec3,
    pub view_up: glam::Vec3,
}

impl PerspectiveCamera {
    const INIT_FORWARD: Vec3 = Vec3::new(0.0, 0.0, -1.0);
    const INIT_UP: Vec3 = Vec3::new(0.0, 1.0, 0.0);

    pub fn new(
        persp: PerspectiveProperties,
    ) -> Self {
        let view_forward = Self::INIT_FORWARD;
        let view_up = Self::INIT_UP;

        Self {
            camera_position: Vec3::ZERO,
            perspective_props: persp,
            view_forward,
            view_up,
        }
    }

    pub fn update_perspective_props(
        &mut self,
        size: Option<UVec2>,
        z_value: Option<Vec2>,
        fov: Option<f32>,
    ) {
        if let Some(size) = size {
            self.perspective_props.aspect_ratio = (size.x as f32) / (size.y as f32);
        }
        if let Some(z_value) = z_value {
            self.perspective_props.z_near = z_value.x;
            self.perspective_props.z_far = z_value.y;
        }
        if let Some(fov) = fov {
            self.perspective_props.fov_radians = fov;
        }
    }

    pub fn generate_camera_matrices(&mut self) -> CameraMatrices {
        let view_matrix = glam::Mat4::look_at_rh(self.camera_position, self.camera_position + self.view_forward, self.view_up);
        let transform_matrix = view_matrix.inverse();
        let projection_matrix =glam::Mat4::perspective_rh_gl(
            self.perspective_props.fov_radians,
            self.perspective_props.aspect_ratio,
            self.perspective_props.z_near,
            self.perspective_props.z_far,
        );
        let projection_view_matrix = projection_matrix.mul_mat4(&view_matrix);
        CameraMatrices {
            transform_matrix,
            view_matrix,
            projection_matrix,
            projection_view_matrix,
        }
    }
}

use crate::Orientable;

use super::CameraMatrices;

impl Orientable for PerspectiveCamera {
    fn rotate(&mut self, rotation: glam::Quat) {
        self.view_forward = rotation.mul_vec3(self.view_forward);
        self.view_up = rotation.mul_vec3(self.view_up);
    }

    fn set_orientation(&mut self, orientation: glam::Quat) {
        self.view_forward = orientation.mul_vec3(Self::INIT_FORWARD);
        self.view_up = orientation.mul_vec3(Self::INIT_UP);
    }

    fn get_orientation(&self) -> glam::Quat {
        glam::Quat::from_rotation_arc(Self::INIT_FORWARD, self.view_forward)
    }
}
