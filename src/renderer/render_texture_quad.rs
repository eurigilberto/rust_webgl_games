use std::{rc::Rc, cell::RefCell};

use glam::Mat4;
use rust_webgl2::{
    DrawCapabilities, GlMaterial, GlTexture2D, Graphics, ShaderAttribute, ShaderSource,
    ShaderStage, ShaderUniform, ShaderVarying, UniformCollection, WebGLDataType, UniformIndex, PrimitiveType,
};

use crate::camera::get_camera_uniform_block_definition;

use super::Renderer;

const VERTEX_MAIN: &str = r#"
vec2 positions[4] = vec2[](
	vec2(-0.5, 0.5),
	vec2(0.5, 0.5),
	vec2(-0.5, -0.5),
	vec2(0.5, -0.5)
);
v_uv = positions[gl_VertexID] + vec2(0.5, 0.5);
vec3 v_position = vec3(positions[gl_VertexID].x, positions[gl_VertexID].y, 0);
vec4 world_position = m_transform * vec4(v_position, 1.0);
vec4 clip_position = proj_x_view * world_position;
gl_Position = clip_position;
"#;

pub fn get_quad_shader(fragment_shader: String) -> ShaderSource {
    ShaderSource {
        name: "Quad Texture Render".into(),
        varyings: vec![ShaderVarying {
            interp: None,
            kind: WebGLDataType::Vec2,
            name: "v_uv".into(),
        }],
        common_uniforms: UniformCollection {
            uniforms: vec![
                ShaderUniform {
                    array_length: None,
                    kind: WebGLDataType::Mat4,
                    name: "m_transform".into(),
                },
                ShaderUniform {
                    array_length: None,
                    kind: WebGLDataType::Sampler2D,
                    name: "u_texture".into(),
                },
            ],
            uniform_blocks: vec![get_camera_uniform_block_definition()],
        },
        imported_functions: vec![],
        vertex_shader: ShaderStage {
            import_fn: vec![],
            main_fn: VERTEX_MAIN.into(),
            attributes: vec![],
            uniform_collection: UniformCollection::new(),
        },
        fragment_shader: ShaderStage {
            import_fn: vec![],
            main_fn: fragment_shader,
            attributes: vec![ShaderAttribute::get_default_frag_attribute()],
            uniform_collection: UniformCollection::new(),
        },
        local_import: None
    }
}

pub struct RenderTextureQuad {
    material: Rc<RefCell<GlMaterial>>,
    transform_index: UniformIndex,
    transform: Mat4,
}

impl RenderTextureQuad {
    pub fn new(graphics: &Graphics, fragment_shader: String, texture_ref: Rc<GlTexture2D>) -> Self {
        let mut material = GlMaterial::with_source(
            graphics,
            vec![DrawCapabilities::default_opaque_no_cull()],
            &get_quad_shader(fragment_shader),
        )
        .unwrap();
        material.set_texture_sampler_uniform("u_texture", texture_ref).expect("Failed to set texture");
        let transform_index = material.insert_uniform(Mat4::IDENTITY, "m_transform".into());
        Self {
            material: Rc::new(RefCell::new(material)),
            transform_index,
            transform: Mat4::IDENTITY,
        }
    }

    pub fn set_texture(&mut self, texture_ref: Rc<GlTexture2D>) {
        self.material.borrow_mut()
            .swap_texture_sampler_uniform("u_texture", texture_ref)
            .unwrap();
    }

	pub fn set_transform(&mut self, transform: Mat4){
		self.transform = transform;
		self.material.borrow_mut().set_uniform_value(self.transform_index, transform);
	}

	pub fn request_render(&self, renderer: &Renderer){
		let mat = Rc::clone(&self.material);
		renderer.insert_render_request(Box::new(move |graphics, _|{
			let mut material = mat.borrow_mut();
			material.set_capabilities(graphics, 0);
			material.push_texture_samplers(graphics);
            let mut current_program = material.program.use_program();
            current_program.push_all_uniforms();
            current_program.draw_arrays(PrimitiveType::TRIANGLE_STRIP, 0, 4);
		}), 0);
	}
}
