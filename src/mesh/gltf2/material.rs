use std::{cell::RefCell, rc::Rc};

use crate::{set_camera_uniform_block_binding};

use super::{shader::default_shader};
use glam::*;
use rust_webgl2::*;

pub struct Gltf2DefaultMaterial {
    pub material: Rc<RefCell<GlMaterial>>,
    pub property_index: PropertyIndex,
}

pub struct PropertyIndex {
    pub model_transform: UniformIndex,
    pub light_direction: UniformIndex,
    pub light_color: UniformIndex,
    pub albedo: UniformIndex,
    pub metallic: UniformIndex,
    pub roughness: UniformIndex,
}

impl Gltf2DefaultMaterial {
    pub fn new(graphics: &Graphics) -> Result<Self, String> {

        let draw_capabilities = vec![DrawCapabilities {
            cull_face: Some(CullMode::BACK),
            depth_test: Some(DepthFunction::LEQUAL),
            ..Default::default()
        }];

        let shader_source = default_shader();
        let mut material = GlMaterial::with_source(graphics, draw_capabilities, &shader_source).expect("Material creation error");
        set_camera_uniform_block_binding(&material.program);

        //let common_uniforms = &shader_source.common_uniforms.uniforms;
        //let res_global_uniforms = material.program.insert_shader_uniforms(common_uniforms);
        //let vertex_uniforms = &shader_source.vertex_shader.uniform_collection.uniforms;
        //let res_vertex_uniforms = material.program.insert_shader_uniforms(vertex_uniforms);
        
        let property_index = {
            PropertyIndex {
                model_transform: material.insert_uniform(Mat4::IDENTITY, "model_transform"),
                light_direction: material.insert_uniform(Vec3::ZERO, "light_dir"),
                light_color: material.insert_uniform(RGBA::WHITE, "light_color"),
                albedo: material.insert_uniform(RGBA::BLACK, "albedo"),
                metallic: material.insert_uniform(0.0 as f32, "metallic"),
                roughness: material.insert_uniform(0.0 as f32, "roughness"),
            }
        };

        Ok(Self{
            material: Rc::new(RefCell::new(material)),
            property_index,
        })
    }
}

//Setting material parameter
impl Gltf2DefaultMaterial {
    pub fn set_model_transform(&self, transform: Mat4) {
        self.material.borrow_mut().program.uniforms.set_uniform(
            self.property_index.model_transform,
            FloatUniform::Mat4(transform).into(),
        );
    }

    pub fn set_light(&self, direciton: Vec3, color: RGBA) {
        self.material.borrow_mut().program.uniforms.set_uniform(
            self.property_index.light_direction,
            FloatUniform::Vec3(direciton).into(),
        );
        self.material.borrow_mut().program.uniforms.set_uniform(
            self.property_index.light_color,
            FloatUniform::Vec3(vec3(color.r, color.g, color.b)).into(),
        );
    }

    pub fn set_albedo(&self, color: RGBA) {
        let c = vec3(color.r, color.g, color.b);
        self.material.borrow_mut().program
            .uniforms
            .set_uniform(self.property_index.albedo, FloatUniform::Vec3(c).into());
    }

    pub fn set_metallic(&self, value: f32) {
        self.material.borrow_mut().program.uniforms.set_uniform(
            self.property_index.metallic,
            FloatUniform::Scalar(value).into(),
        )
    }

    pub fn set_roughness(&self, value: f32) {
        self.material.borrow_mut().program.uniforms.set_uniform(
            self.property_index.roughness,
            FloatUniform::Scalar(value).into(),
        )
    }
}
