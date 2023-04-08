use rust_webgl2::*;

use crate::camera::get_camera_uniform_block_definition;

const VERT_MAIN_FN: &str = r#"
vec4 world_position = model_transform * vec4(att_position, 1.0);
v_position = world_position.xyz;
v_normal = normalize((model_transform * vec4(att_normal, 0.0)).xyz);
cam_position = camera_transform[3].xyz;
gl_Position = proj_x_view * world_position;
"#;

const FRAG_MAIN_FN: &str = r#"
vec3 normal = normalize(v_normal);
vec3 world_pos = v_position;
vec3 w_pos_to_light = normalize(-light_dir);

vec3 L_sun = PBRLighting(normal, cam_position, world_pos, w_pos_to_light, albedo, metallic, roughness, light_color);

vec3 ambient = vec3(0.03) * albedo;
vec3 color = ambient + L_sun;

color = color / (color + vec3(1.0));
color = pow(color, vec3(1.0/2.2));

frag_color = vec4(color, 1.0);
"#;

pub fn default_shader() -> ShaderSource {
    let mut imported_functions = Vec::new();
    imported_functions.extend(lighting_functions::get_pbr_functions());

    ShaderSource {
        name: String::from("Mesh Default Shader"),
        varyings: vec![
            ShaderVarying {
                interp: None,
                kind: WebGLDataType::Vec3,
                name: "v_position".into(),
            },
            ShaderVarying {
                interp: None,
                kind: WebGLDataType::Vec3,
                name: "v_normal".into(),
            },
            ShaderVarying {
                interp: None,
                kind: WebGLDataType::Vec3,
                name: "cam_position".into(),
            },
        ],
        common_uniforms: UniformCollection {
            uniforms: vec![
                ShaderUniform {
                    array_length: None,
                    kind: WebGLDataType::Vec3,
                    name: "albedo".into(),
                },
                ShaderUniform {
                    array_length: None,
                    kind: WebGLDataType::Float,
                    name: "metallic".into(),
                },
                ShaderUniform {
                    array_length: None,
                    kind: WebGLDataType::Float,
                    name: "roughness".into(),
                },
                ShaderUniform {
                    array_length: None,
                    kind: WebGLDataType::Vec3,
                    name: "light_dir".into(),
                },
                ShaderUniform {
                    array_length: None,
                    kind: WebGLDataType::Vec3,
                    name: "light_color".into(),
                },
            ],
            uniform_blocks: Vec::new(),
        },
        imported_functions: imported_functions,
        vertex_shader: ShaderStage {
            attributes: vec![
                ShaderAttribute {
                    layout_loc: 0,
                    kind: WebGLDataType::Vec3,
                    name: "att_position".into(),
                },
                ShaderAttribute {
                    layout_loc: 1,
                    kind: WebGLDataType::Vec3,
                    name: "att_normal".into(),
                },
            ],
            import_fn: Vec::new(),
            uniform_collection: UniformCollection {
                uniforms: vec![ShaderUniform {
                    array_length: None,
                    kind: WebGLDataType::Mat4,
                    name: "model_transform".into(),
                }],
                uniform_blocks: vec![get_camera_uniform_block_definition()],
            },
            main_fn: VERT_MAIN_FN.into(),
        },
        fragment_shader: ShaderStage {
            attributes: vec![ShaderAttribute {
                layout_loc: 0,
                kind: WebGLDataType::Vec4,
                name: "frag_color".into(),
            }],
            import_fn: vec![
                "fresnelSchlick".into(),
                "DistributionGGX".into(),
                "GeometrySchlickGGX".into(),
                "GeometrySmith".into(),
                "PBRLighting".into(),
            ],
            uniform_collection: UniformCollection {
                uniforms: Vec::new(),
                uniform_blocks: Vec::new(),
            },
            main_fn: FRAG_MAIN_FN.into(),
        },
        local_import: None
    }
}
