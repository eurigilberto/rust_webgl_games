use rust_webgl2::*;

use crate::camera::get_camera_uniform_block_definition;

use super::CubeGizmoInstanceData;

const VERT_MAIN_FN: &str = r#"
vec4 tr0 = tr_row_0;
vec4 tr1 = tr_row_1;
vec4 tr2 = tr_row_2;

cube_scale = vec3(length(tr0.xyz), length(tr1.xyz), length(tr2.xyz));

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
vec3 outline_size = vec3(0.1) / cube_scale.xyz;
vec3 mask_size = vec3(0.5) - (outline_size * 0.5);
vec3 abs_pos = abs(os_position);
vec2 xz_mask = step(abs_pos.xz, mask_size.xz);
vec2 xy_mask = step(abs_pos.xy, mask_size.xy);
vec2 yz_mask = step(abs_pos.yz, mask_size.yz);

vec3 out_size = vec3(0.5) - outline_size * 0.05;
vec3 step_mask = step(out_size, abs_pos);

float y_mult = 1.0 + step_mask.y + step_mask.x * 2.0;
y_mult = (step_mask.y * step_mask.z) + (step_mask.y * step_mask.x) + (step_mask.z * step_mask.x);//1.0 / y_mult;
y_mult = 1.0 - clamp(y_mult, 0.0, 1.0);

float mask = (xz_mask.x * xz_mask.y) + (xy_mask.x * xy_mask.y) + (yz_mask.x * yz_mask.y);
mask = clamp(mask, 0.0, 1.0);
vec3 color = inst_color.xyz * mix(0.75, 1.0, mask);
frag_color = vec4(color * y_mult, 1.0);"#;

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
            ShaderVarying {
                interp: None,
                kind: WebGLDataType::Vec3,
                name: "cube_scale".into(),
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
        local_import: None
    }
}
