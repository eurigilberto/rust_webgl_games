use std::rc::Rc;

use rust_webgl2::FunctionDefinition;
use rust_webgl2::FunctionDefinitionType;
use rust_webgl2::GlTexture2D;
use rust_webgl2::Graphics;
use rust_webgl2::ShaderAttribute;
use rust_webgl2::ShaderStage;
use rust_webgl2::ShaderUniform;
use rust_webgl2::UniformCollection;
use rust_webgl2::WebGLDataType;

use crate::renderer::texture_render::ColorRenderable;
use crate::renderer::texture_render::TextureShaderRender;
use crate::renderer::texture_render::TextureShaderSource;

pub fn get_pixel_distance() -> FunctionDefinition {
    FunctionDefinition {
        name: "pixel_distance".into(),
        definition: FunctionDefinitionType::InlineFn {
            return_type: WebGLDataType::Float,
            parameters: "ivec2 center, ivec2 pixel, int max_range".into(),
            body: r#"
ivec2 delta = pixel - center;
return clamp(length(vec2(delta.x, delta.y)) / float(max_range), 0.0, 1.0);
"#
            .into(),
        },
    }
}

pub fn get_texture_texel() -> FunctionDefinition {
    FunctionDefinition {
        name: "fetch_pixel".into(),
        definition: FunctionDefinitionType::InlineFn {
            return_type: WebGLDataType::Int,
            parameters: "sampler2D i_texture, ivec2 uv".into(),
            body: r#"
vec4 texel = texelFetch(i_texture, uv, 0);
if(texel.x > 0.5){
	return 1;
}else{
	return 0;
}"#
            .into(),
        },
    }
}

const FRAGMENT_MAIN: &str = r#"
ivec2 px_uv = ivec2(gl_FragCoord.xy);

int value = fetch_pixel(input_texture, px_uv);
float min_distance = 1.0;
for(int i = -distance_range; i <= distance_range; i++) {
	for(int j = -distance_range; j <= distance_range; j++) {
		ivec2 n_uv = px_uv + ivec2(i, j);
		int n_value = fetch_pixel(input_texture, n_uv);
		if(value != n_value) {
			float distance = length(vec2(float(i), float(j)));
			distance = clamp(distance / float(distance_range), 0.0, 1.0);
			min_distance = min(min_distance, distance);
		}
	}
}

float distance = min_distance;
if (value == 1) {
	distance *= -1.0;
}
float remapped = (distance + 1.0) / 2.0;
frag_color = vec4(remapped);
"#;

pub struct SDFGeneration {
    texture_render: TextureShaderRender,
}

impl SDFGeneration {
    /// Creates a new SDF Generation shader
    /// # Arguments
    /// * `graphics` - The graphics context
    /// * `texture_ref` - The texture to generate the SDF for
    /// * `distance_range` - The distance range to check for in pixels
    pub fn new(graphics: &Graphics, texture_ref: Rc<GlTexture2D>, distance_range: u32) -> Self {
        let mut texture_render = TextureShaderRender::new(
            graphics,
            texture_ref.props,
            texture_ref.size,
            vec![ColorRenderable::R8.into()],
            TextureShaderSource {
                name: "SDF_Generation".into(),
                imported_functions: vec![get_texture_texel(), get_pixel_distance()],
                fragment_shader: ShaderStage {
                    import_fn: vec!["fetch_pixel".into(), "pixel_distance".into()],
                    main_fn: FRAGMENT_MAIN.into(),
                    attributes: vec![ShaderAttribute::get_default_frag_attribute()],
                    uniform_collection: UniformCollection {
                        uniforms: vec![
                            ShaderUniform {
                                array_length: None,
                                kind: WebGLDataType::Sampler2D,
                                name: "input_texture".into(),
                            },
                            ShaderUniform {
                                array_length: None,
                                kind: WebGLDataType::Int,
                                name: "distance_range".into(),
                            },
                        ],
                        uniform_blocks: vec![],
                    },
                },
            },
            Some("SDF_Texture".into())
        )
        .expect("Texture Shader Render creation failed");
        texture_render
            .material
            .insert_uniform(distance_range as i32, "distance_range");
        texture_render
            .material
            .set_texture_sampler_uniform("input_texture", texture_ref)
            .unwrap();
        Self { texture_render }
    }

    pub fn get_texture_ref(&self) -> Rc<GlTexture2D> {
        self.texture_render.framebuffer.get_texture_ref(0)
    }

    pub fn render_texture(&mut self, graphics: &Graphics) {
        self.texture_render.render_texture(graphics);
    }
}
