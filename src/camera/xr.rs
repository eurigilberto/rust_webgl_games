use glam::*;

use super::CameraMatrices;

pub struct XrCamera {}

impl XrCamera {
    pub fn new() -> Self {
        Self {}
    }

    pub fn generate_camera_matrices(
        &mut self,
        proj_matrix: Vec<f32>,
        cam_matrix: Vec<f32>,
        camera_offset: Vec3,
    ) -> CameraMatrices {
        let mut proj_matrix_arr = [0.0; 16];

        for (index, val) in proj_matrix.iter().enumerate() {
            proj_matrix_arr[index] = *val;
        }
        let proj_matrix = Mat4::from_cols_array(&proj_matrix_arr);

        let mut cam_matrix_arr = [0.0; 16];
        for (index, val) in cam_matrix.iter().enumerate() {
            cam_matrix_arr[index] = *val;
        }
        let player_camera_offset = glam::Mat4::from_translation(camera_offset);
        let cam_matrix = player_camera_offset.mul_mat4(&Mat4::from_cols_array(&cam_matrix_arr));

        let view_matrix = cam_matrix.inverse();

        let proj_x_view = proj_matrix.mul_mat4(&view_matrix);

        CameraMatrices {
            transform_matrix: cam_matrix,
            view_matrix,
            projection_matrix: proj_matrix,
            projection_view_matrix: proj_x_view,
        }
    }
}
