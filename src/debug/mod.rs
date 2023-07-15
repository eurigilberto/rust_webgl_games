use crate::{gizmos::CubeGizmo, renderer::Renderer};
use glam::*;
use rust_webgl2::*;

pub struct DebugCubeGizmoRenderer {
    cube_gizmo: CubeGizmo,
    gizmo_colors: Vec<RGBA>,
    gizmo_matrix: Vec<Mat4>,
}

impl DebugCubeGizmoRenderer {
    pub fn new(renderer: &mut Renderer, gizmo_count: u32) -> Self {
        let cube_gizmo = CubeGizmo::new(renderer.get_graphics(), gizmo_count);

        Self {
            cube_gizmo,
            gizmo_colors: Vec::new(),
            gizmo_matrix: Vec::new()
        }
    }

    pub fn push_cube(&mut self, pos: Vec3, size: Vec3, color: RGBA) {
        self.gizmo_colors.push(color);
        self.gizmo_matrix.push(Mat4::from_scale_rotation_translation(size, Quat::IDENTITY, pos));
    }

    pub fn push_flat_cube(&mut self, pos: Vec3, size: f32, height: f32, color: RGBA) {
        self.gizmo_colors.push(color);
        let size = vec3(size, height, size);
        self.gizmo_matrix.push(Mat4::from_scale_rotation_translation(size, Quat::IDENTITY, pos));
    }

    pub fn push_cube_matrix(&mut self, matrix: Mat4, color: RGBA){
        self.gizmo_colors.push(color);
        self.gizmo_matrix.push(matrix);
    }

    pub fn clear_gizmos(&mut self) {
        self.gizmo_colors.clear();
        self.gizmo_matrix.clear();
    }

    pub fn get_current_count(&self) -> usize {
        self.gizmo_matrix.len()
    }

    pub fn render(&mut self, renderer: &Renderer, render_layer: usize) {
        if self.gizmo_matrix.len() > 0 {
            self.cube_gizmo
                .update_instance_data(&self.gizmo_matrix, &self.gizmo_colors);
            self.cube_gizmo.request_mutliple_renders(renderer, render_layer);
        }
    }
}

pub mod cube_debug_render {
    use glam::{Vec3, vec3, Mat4};
    use rust_webgl2::*;

    use crate::renderer::Renderer;

    use super::DebugCubeGizmoRenderer;

    pub static mut DEBUG_CUBE_RENDERER: Option<DebugCubeGizmoRenderer> = None;

    pub fn create_debug_renderer(renderer: &mut Renderer, gizmo_count: u32) {
        unsafe { DEBUG_CUBE_RENDERER = Some(DebugCubeGizmoRenderer::new(renderer, gizmo_count)) }
    }

    pub fn push_debug_cube(pos: Vec3, size: f32, color: RGBA) {
        unsafe {
            if let Some(cube_renderer) = DEBUG_CUBE_RENDERER.as_mut() {
                cube_renderer.push_cube(pos, vec3(size,size,size), color);
            }
        }
    }

    pub fn push_debug_cuboid(pos: Vec3, size: Vec3, color: RGBA) {
        unsafe {
            if let Some(cube_renderer) = DEBUG_CUBE_RENDERER.as_mut() {
                cube_renderer.push_cube(pos, size, color);
            }
        }
    }

    pub fn push_debug_flat_cube(pos: Vec3, size: f32, height: f32, color: RGBA) {
        unsafe {
            if let Some(cube_renderer) = DEBUG_CUBE_RENDERER.as_mut() {
                cube_renderer.push_flat_cube(pos, size, height, color);
            }
        }
    }

    pub fn push_debug_cube_matrix(matrix: Mat4, color: RGBA){
        unsafe{
            if let Some(cube_renderer) = DEBUG_CUBE_RENDERER.as_mut(){
                cube_renderer.push_cube_matrix(matrix, color);
            }
        }
    }

    pub fn clear() {
        unsafe {
            if let Some(cube_renderer) = DEBUG_CUBE_RENDERER.as_mut() {
                cube_renderer.clear_gizmos();
            }
        }
    }

    pub fn request_render(renderer: &Renderer, render_layer: usize) {
        unsafe { 
            if let Some(cube_renderer) = DEBUG_CUBE_RENDERER.as_mut() {
                cube_renderer.render(renderer, render_layer);
            }
        }
    }
}
