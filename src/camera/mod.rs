mod perspective;
use glam::Mat4;
pub use perspective::*;
use rust_webgl2::*;

mod xr;
pub use xr::*;

#[derive(Clone, Copy)]
pub struct CameraMatrices {
    pub transform_matrix: Mat4,
    pub view_matrix: Mat4,
    pub projection_matrix: Mat4,
    pub projection_view_matrix: Mat4,
}

impl CameraMatrices{
    pub fn identity()->Self{
        Self{
            transform_matrix: Mat4::IDENTITY,
            view_matrix: Mat4::IDENTITY,
            projection_matrix: Mat4::IDENTITY,
            projection_view_matrix: Mat4::IDENTITY,
        }
    }
}

pub struct ViewData {
    pub camera_matrices: CameraMatrices,
    pub viewport: Viewport,
}

pub fn create_camera_uniform_buffer(graphics: &Graphics) -> GlUniformBuffer {
    GlUniformBuffer::with_capacity(
        graphics,
        16 * 3 * 4,
        BufferUsage::DYNAMIC_DRAW,
        crate::CAMERA_BINDING_NUMBER,
    )
    .unwrap()
}

pub fn update_camera_buffer(camera_matrices: CameraMatrices, camera_buffer: &mut GlUniformBuffer) {
    let mut view_buffer = Vec::new();
    view_buffer.extend(camera_matrices.transform_matrix.to_cols_array());
    view_buffer.extend(camera_matrices.view_matrix.to_cols_array());
    view_buffer.extend(camera_matrices.projection_matrix.to_cols_array());
    view_buffer.extend(camera_matrices.projection_view_matrix.to_cols_array());

    camera_buffer.buffer_data(view_buffer.as_slice());
}

pub fn get_camera_uniform_block_definition() -> ShaderUniformBlock {
    ShaderUniformBlock {
        name: "ViewMatrices".into(),
        binding_number: crate::CAMERA_BINDING_NUMBER,
        uniforms: vec![
            ShaderUniform {
                array_length: None,
                kind: WebGLDataType::Mat4,
                name: "camera_transform".into(),
            },
            ShaderUniform {
                array_length: None,
                kind: WebGLDataType::Mat4,
                name: "view_matrix".into(),
            },
            ShaderUniform {
                array_length: None,
                kind: WebGLDataType::Mat4,
                name: "view_projection".into(),
            },
            ShaderUniform {
                array_length: None,
                kind: WebGLDataType::Mat4,
                name: "proj_x_view".into(),
            },
        ],
    }
}
