use rust_webgl2::*;

use crate::camera::get_camera_uniform_block_definition;

use super::CubeGizmoInstanceData;

const VERT_MAIN_FN: &str = r#"
vec4 tr0 = tr_row_0;
vec4 tr1 = tr_row_1;
vec4 tr2 = tr_row_2;

vec4 tc0 = vec4(tr0.x, tr1.x, tr2.x, 0.0);
vec4 tc1 = vec4(tr0.y, tr1.y, tr2.y, 0.0);
vec4 tc2 = vec4(tr0.z, tr1.z, tr2.z, 0.0);
vec4 tc3 = vec4(tr0.w, tr1.w, tr2.w, 1.0);

mat4 cube_transform = mat4(tc0, tc1, tc2, tc3);
inst_color = c_color;

vec4 world_position = cube_transform * vec4(vert_pos, 1.0);
vec4 clip_position = proj_x_view * world_position;
ws_position = world_position.xyz;
os_position = vert_pos;
gl_Position = clip_position;"#;

const FRAG_MAIN_FN: &str = r#"
float y_param = (os_position.y * 0.5 + 0.5);
vec3 color = mix(0.5, 1.0, y_param) * inst_color.xyz;
frag_color = vec4(color, 1.0);"#;

/*pub struct DefaultGizmoUnifomrs {
    pub model_transform_index: UniformIndex,
    pub gizmo_color_index: UniformIndex,
}
pub fn get_default_gizmo_uniforms(material: &mut GlMaterial) -> DefaultGizmoUnifomrs {
    let model_transform_index = material
        .program
        .insert_uniform(
            "model_transform".into(),
            GlUniform::Float(FloatUniform::Mat4(Mat4::IDENTITY)),
        )
        .unwrap();
    let gizmo_color_index = material
        .program
        .insert_uniform(
            "gizmo_color".into(),
            GlUniform::Float(FloatUniform::Vec4(vec4(0.0, 0.0, 0.0, 1.0))),
        )
        .unwrap();
    DefaultGizmoUnifomrs {
        model_transform_index,
        gizmo_color_index,
    }
}*/

pub fn gizmo_default_shader_source() -> ShaderSource {
    let mut vertex_attribs = Vec::new();
    vertex_attribs.push(ShaderAttribute {
        layout_loc: 0,
        kind: WebGLDataType::Vec3,
        name: "vert_pos".into(),
    });
    vertex_attribs.extend(CubeGizmoInstanceData::get_shader_attributes(1).into_iter());
    ShaderSource {
        name: "GizmoShader".into(),
        varyings: vec![
            ShaderVarying {
                interp: None,
                kind: WebGLDataType::Vec3,
                name: "ws_position".into(),
            },
            ShaderVarying {
                interp: None,
                kind: WebGLDataType::Vec3,
                name: "os_position".into(),
            },
            ShaderVarying {
                interp: None,
                kind: WebGLDataType::Vec4,
                name: "inst_color".into(),
            },
        ],
        common_uniforms: UniformCollection {
            uniforms: vec![],
            uniform_blocks: Vec::new(),
        },
        imported_functions: Vec::new(),
        vertex_shader: ShaderStage {
            import_fn: Vec::new(),
            main_fn: VERT_MAIN_FN.into(),
            attributes: vertex_attribs,
            uniform_collection: UniformCollection {
                uniforms: vec![],
                uniform_blocks: vec![get_camera_uniform_block_definition()],
            },
        },
        fragment_shader: ShaderStage {
            import_fn: Vec::new(),
            main_fn: FRAG_MAIN_FN.into(),
            attributes: vec![ShaderAttribute {
                layout_loc: 0,
                kind: WebGLDataType::Vec4,
                name: "frag_color".into(),
            }],
            uniform_collection: UniformCollection {
                uniforms: Vec::new(),
                uniform_blocks: Vec::new(),
            },
        },
    }
}
