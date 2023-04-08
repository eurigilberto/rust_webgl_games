use glam::*;
use rust_webgl2::*;

use crate::{camera::get_camera_uniform_block_definition, set_camera_uniform_block_binding};

pub fn get_curve_2d_shader() -> ShaderSource {
    ShaderSource {
        name: "line 2d shader".into(),
        varyings: vec![ShaderVarying {
            interp: None,
            kind: WebGLDataType::Vec2,
            name: "frag_uv".into(),
        }],
        common_uniforms: UniformCollection {
            uniforms: vec![ShaderUniform {
                array_length: None,
                kind: WebGLDataType::Vec3,
                name: "line_color".into(),
            }],
            uniform_blocks: Vec::new(),
        },
        imported_functions: Vec::new(),
        vertex_shader: ShaderStage {
            import_fn: vec![],
            main_fn: r#"
vec2 pos = position_direction.xy;
vec2 dir = position_direction.zw;

vec3 os_center_pos = vec3(pos.x, 0.0, pos.y);
vec3 os_dir = vec3(dir.x, 0.0, dir.y);

frag_uv = vert_uv;

vec3 position = os_center_pos + os_dir * half_line_width;
vec4 world_position = model_transform * vec4(position, 1.0);
vec4 clip_position = proj_x_view * world_position;

gl_Position = clip_position;
"#
            .into(),
            attributes: vec![
                ShaderAttribute {
                    layout_loc: 0,
                    kind: WebGLDataType::Vec4,
                    name: "position_direction".into(),
                },
                ShaderAttribute {
                    layout_loc: 1,
                    kind: WebGLDataType::Vec2,
                    name: "vert_uv".into(),
                },
            ],
            uniform_collection: UniformCollection {
                uniforms: vec![
                    ShaderUniform {
                        array_length: None,
                        kind: WebGLDataType::Mat4,
                        name: "model_transform".into(),
                    },
                    ShaderUniform {
                        array_length: None,
                        kind: WebGLDataType::Float,
                        name: "half_line_width".into(),
                    },
                ],
                uniform_blocks: vec![get_camera_uniform_block_definition()],
            },
        },
        fragment_shader: ShaderStage {
            import_fn: vec![],
            main_fn: r#"
frag_color = vec4(line_color, 1.0);
            "#
            .into(),
            attributes: vec![ShaderAttribute {
                layout_loc: 0,
                kind: WebGLDataType::Vec4,
                name: "frag_color".into(),
            }],
            uniform_collection: UniformCollection::new(),
        },
        local_import: None,
    }
}

pub struct DefaultCurve2DMaterial {
    pub material: GlMaterial,
    pub line_color: UniformIndex,
    pub half_line: UniformIndex,
    pub model_transform: UniformIndex,
}

impl DefaultCurve2DMaterial {
    pub fn new(graphics: &Graphics, line_color: RGBA, line_width: f32, transform: Mat4) -> Self {
        let mut material = GlMaterial::with_source(
            graphics,
            vec![DrawCapabilities::default_opaque()],
            &get_curve_2d_shader(),
        ).expect("Material creation error");
        let line_color_index = material
            .program
            .insert_uniform(
                "line_color".into(),
                GlUniform::Float(FloatUniform::Vec3(Vec3::from_array(line_color.into()))),
            )
            .unwrap();
        let half_line_width_index = material
            .program
            .insert_uniform(
                "half_line_width",
                GlUniform::Float(FloatUniform::Scalar(line_width)),
            )
            .unwrap();
        let model_transform_index = material
            .program
            .insert_uniform(
                "model_transform",
                GlUniform::Float(FloatUniform::Mat4(transform)),
            )
            .unwrap();
        set_camera_uniform_block_binding(&material.program);
        Self {
            material,
            line_color: line_color_index,
            half_line: half_line_width_index,
            model_transform: model_transform_index,
        }
    }
}