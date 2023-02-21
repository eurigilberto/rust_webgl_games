use crate::{gizmos::CubeGizmo, renderer::Renderer};
use glam::*;
use rust_webgl2::*;

pub struct DebugCubeGizmoRenderer {
    cube_gizmo: CubeGizmo,
    gizmo_positions: Vec<Vec3>,
    gizmo_sizes: Vec<Vec3>,
    gizmo_colors: Vec<RGBA>,
}

impl DebugCubeGizmoRenderer {
    pub fn new(renderer: &mut Renderer) -> Self {
        let cube_gizmo = CubeGizmo::new(renderer.get_graphics(), 2500);

        Self {
            cube_gizmo,
            gizmo_positions: Vec::new(),
            gizmo_sizes: Vec::new(),
            gizmo_colors: Vec::new(),
        }
    }

    pub fn push_cube(&mut self, pos: Vec3, size: Vec3, color: RGBA) {
        self.gizmo_positions.push(pos);
        self.gizmo_colors.push(color);
        self.gizmo_sizes.push(size);
    }

    pub fn push_flat_cube(&mut self, pos: Vec3, size: f32, height: f32, color: RGBA) {
        self.gizmo_positions.push(pos);
        self.gizmo_colors.push(color);
        self.gizmo_sizes.push(vec3(size, height, size));
    }

    pub fn clear_gizmos(&mut self) {
        self.gizmo_colors.clear();
        self.gizmo_positions.clear();
        self.gizmo_sizes.clear();
    }

    pub fn get_current_count(&self) -> usize {
        self.gizmo_positions.len()
    }

    pub fn render(&mut self, renderer: &Renderer) {
        if self.gizmo_positions.len() > 0 {
            let transforms = (self.gizmo_positions.iter().zip(self.gizmo_sizes.iter()))
                .map(|(pos, size)| {
                    Mat4::from_scale_rotation_translation(*size, Quat::IDENTITY, *pos)
                })
                .collect();
            self.cube_gizmo
                .update_instance_data(&transforms, &self.gizmo_colors);
            self.cube_gizmo.request_mutliple_renders(renderer);
        }
    }
}

pub mod cube_debug_render {
    use glam::{Vec3, vec3};
    use rust_webgl2::*;

    use crate::renderer::Renderer;

    use super::DebugCubeGizmoRenderer;

    pub static mut DEBUG_CUBE_RENDERER: Option<DebugCubeGizmoRenderer> = None;

    pub fn create_debug_renderer(renderer: &mut Renderer) {
        unsafe { DEBUG_CUBE_RENDERER = Some(DebugCubeGizmoRenderer::new(renderer)) }
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

    pub fn clear() {
        unsafe {
            if let Some(cube_renderer) = DEBUG_CUBE_RENDERER.as_mut() {
                cube_renderer.clear_gizmos();
            }
        }
    }

    pub fn request_render(renderer: &Renderer) {
        unsafe { 
            if let Some(cube_renderer) = DEBUG_CUBE_RENDERER.as_mut() {
                cube_renderer.render(renderer);
            }
        }
    }
}
