use std::cell::RefCell;
use std::rc::Rc;

use bytemuck::{Pod, Zeroable};
use rust_webgl2::*;

use crate::{geometry, set_camera_uniform_block_binding};
use crate::renderer::Renderer;
use glam::*;

mod material;
use material::*;

const CUBE_INDEX_TYPE: IndexType = IndexType::U16;

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct CubeGizmoInstanceData {
    pub tr_row_0: Vec4,
    pub tr_row_1: Vec4,
    pub tr_row_2: Vec4,
    pub color: Vec4,
}

impl CubeGizmoInstanceData {
    pub const fn get_size() -> u32 {
        ((std::mem::size_of::<f32>() * 4) * 4) as u32
    }
    pub const fn get_stride() -> u8 {
        Self::get_size() as u8
    }
    pub fn get_offset(index: u32) -> u32 {
        (std::mem::size_of::<f32>() as u32 * 4) * index
    }

    pub fn get_attribute_descriptions(
        loc_offset: u32,
        buffer_index: usize,
    ) -> Vec<AttributeDescription> {
        let mut attribs = Vec::new();
        for i in 0..4 {
            attribs.push(AttributeDescription {
                location: loc_offset + i,
                unit_type: NumberType::FLOAT,
                size: AttributeSize::FOUR,
                buffer: buffer_index,
                normalize: false,
                kind: AttributeType::PerInstanceInterleaved {
                    stride: Self::get_stride(),
                    offset: Self::get_offset(i),
                    divisor: 1,
                },
            });
        }
        attribs
    }

    pub fn get_shader_attributes(loc_offset: u32) -> Vec<ShaderAttribute> {
        vec![
            ShaderAttribute {
                layout_loc: loc_offset,
                kind: WebGLDataType::Vec4,
                name: "tr_row_0".into(),
            },
            ShaderAttribute {
                layout_loc: loc_offset + 1,
                kind: WebGLDataType::Vec4,
                name: "tr_row_1".into(),
            },
            ShaderAttribute {
                layout_loc: loc_offset + 2,
                kind: WebGLDataType::Vec4,
                name: "tr_row_2".into(),
            },
            ShaderAttribute {
                layout_loc: loc_offset + 3,
                kind: WebGLDataType::Vec4,
                name: "c_color".into(),
            },
        ]
    }
}

pub struct CubeGizmoBufferData {
    vao: Rc<GlVertexArrayObject>,
    index_count: u32,
    max_instance: u32,
    instance_count: u32,
    instance_buffer: Rc<GlBuffer>,
}

impl CubeGizmoBufferData {
    pub fn new(graphics: &Graphics, instance_count: u32) -> Self {
        let (positions, _, indices) = geometry::cube::rectangular_cuboid_flat(1.0, 1.0, 1.0);

        let pos_buffer = Rc::new(
            GlBuffer::with_data_static_array_buffer(graphics, &positions)
                .expect("Create gizmo mesh"),
        );
        let idx_buffer = Rc::new(
            GlIndexBuffer::with_data(graphics, IndexType::U16, &indices, BufferUsage::STATIC_DRAW)
                .expect("Create gizmo index buffer"),
        );
        let instance_data_buffer = Rc::new(
            GlBuffer::with_size(
                graphics,
                BindingPoint::ARRAY_BUFFER,
                CubeGizmoInstanceData::get_size() * instance_count,
                BufferUsage::DYNAMIC_DRAW,
            )
            .expect("Error creating instance buffer"),
        );

        let mut attribs = Vec::new();
        attribs.push(AttributeDescription {
            location: 0,
            unit_type: NumberType::FLOAT,
            size: AttributeSize::THREE,
            buffer: 0,
            normalize: false,
            kind: AttributeType::Single,
        });
        attribs.extend(CubeGizmoInstanceData::get_attribute_descriptions(1, 1).into_iter());
        let vao = Rc::new(
            GlVertexArrayObject::new(
                graphics,
                attribs,
                &vec![&pos_buffer, &instance_data_buffer],
                Some(idx_buffer),
            )
            .expect("Cannot create gizmo vertex array object"),
        );
        Self {
            vao,
            index_count: (indices.len() * 3) as u32,
            instance_buffer: instance_data_buffer,
            max_instance: instance_count,
            instance_count: 0,
        }
    }

    pub fn push_data_to_instance_buffer(&mut self, data: &Vec<CubeGizmoInstanceData>) {
        if data.len() as u32 >= self.max_instance {
            panic!("Pushing too many instances to the gizmo buffer")
        }
        self.instance_count = data.len() as u32;
        self.instance_buffer.buffer_data(data);
    }
}

pub struct CubeGizmo {
    buffer_data: CubeGizmoBufferData,
    material: Rc<RefCell<GlMaterial>>,
}

pub struct CubeGizmoRenderData {
    vao: std::rc::Weak<GlVertexArrayObject>,
    material: std::rc::Weak<RefCell<GlMaterial>>,
    index_count: u32,
    index_type: IndexType,
    instance_count: u32,
}

impl CubeGizmoRenderData {
    pub fn new(cube_gizmo: &CubeGizmo) -> Self {
        Self {
            vao: Rc::downgrade(&cube_gizmo.buffer_data.vao),
            material: Rc::downgrade(&cube_gizmo.material),
            index_count: cube_gizmo.buffer_data.index_count,
            index_type: CUBE_INDEX_TYPE,
            instance_count: cube_gizmo.buffer_data.instance_count,
        }
    }
}

impl CubeGizmo {
    pub fn new(graphics: &Graphics, instance_count: u32) -> Self {
        let buffer_data = CubeGizmoBufferData::new(graphics, instance_count);
        let draw_capabilities = DrawCapabilities::default_opaque();
        let material = GlMaterial::with_source(
            graphics,
            vec![draw_capabilities],
            &gizmo_default_shader_source(),
        ).expect("Material creation error");
        set_camera_uniform_block_binding(&material.program);
        Self {
            material: Rc::new(RefCell::new(material)),
            buffer_data,
        }
    }

    pub fn update_instance_data(&mut self, transforms: &Vec<Mat4>, colors: &Vec<RGBA>) {
        let mut instance_buffer_data = Vec::new();
        for (transform, color) in transforms.iter().zip(colors.iter()) {
            let instance = CubeGizmoInstanceData {
                tr_row_0: transform.row(0),
                tr_row_1: transform.row(1),
                tr_row_2: transform.row(2),
                color: Vec4::from_array(color.gamma_corrected().into()),
            };
            instance_buffer_data.push(instance);
        }
        self.buffer_data
            .push_data_to_instance_buffer(&instance_buffer_data);
    }

    pub fn request_mutliple_renders(
        &mut self,
        renderer: &Renderer,
        render_layer: usize,
    ) {
        let render_data = CubeGizmoRenderData::new(&self);
        renderer.insert_render_request(
            Box::new(move |gr: &Graphics, _| {
                let mat = render_data.material.upgrade().unwrap();
                mat.borrow().set_capabilities(gr, 0);
                let vao = render_data.vao.upgrade().unwrap();
                vao.bind();
                let mut mat = mat.borrow_mut();
                let current_program = mat.program.use_program();
                current_program.draw_elements_instanced_with_i32(
                    PrimitiveType::TRIANGLES,
                    render_data.index_count,
                    render_data.index_type,
                    0,
                    render_data.instance_count,
                );
                vao.unbind();
            }),
            render_layer,
        );
    }
}
