use std::rc::Rc;
use glam::*;
use rust_webgl2::*;
use crate::{geometry::cube::rectangular_cuboid_flat, renderer::Renderer};

pub struct CubeMesh {
    pub index_count: u32,
	pub index_type: IndexType,
    pub vao: Rc<GlVertexArrayObject>,
}

impl CubeMesh {
    pub fn new(renderer: &Renderer) -> Result<Self, ()> {
        let (position, normal, index_arr) = rectangular_cuboid_flat(1.0, 1.0, 1.0);
        let res_pos_buffer = GlBuffer::with_data(
            &renderer.get_graphics(),
            BindingPoint::ARRAY_BUFFER,
            position.as_slice(),
            BufferUsage::STATIC_DRAW,
        );
        let res_norm_buffer = GlBuffer::with_data(
            &renderer.get_graphics(),
            BindingPoint::ARRAY_BUFFER,
            normal.as_slice(),
            BufferUsage::STATIC_DRAW,
        );
        let res_index_buffer = GlIndexBuffer::with_data(
            &renderer.get_graphics(),
            IndexType::U16,
            index_arr.as_slice(),
            BufferUsage::STATIC_DRAW,
        );

        match (res_pos_buffer, res_norm_buffer, res_index_buffer) {
            (Ok(position_buffer), Ok(normal_buffer), Ok(index_buffer)) => {
                let position_buffer = Rc::new(position_buffer);
                let normal_buffer = Rc::new(normal_buffer);
                let index_buffer = Rc::new(index_buffer);

                let vao = GlVertexArrayObject::new(
                    renderer.get_graphics(),
                    vec![
                        AttributeDescription {
                            location: 0,
                            unit_type: NumberType::FLOAT,
                            size: AttributeSize::THREE,
                            buffer: 0,
                            normalize: false,
                            kind: AttributeType::Single,
                        },
                        AttributeDescription {
                            location: 1,
                            unit_type: NumberType::FLOAT,
                            size: AttributeSize::THREE,
                            buffer: 1,
                            normalize: false,
                            kind: AttributeType::Single,
                        },
                    ],
                    &[&position_buffer, &normal_buffer],
                    Some(index_buffer),
                )
                .unwrap();

                Ok(Self {
					index_count: index_arr.len() as u32 * 3,
                    index_type: IndexType::U16,
                    vao: Rc::new(vao),
                })
            }
            _ => Err(()),
        }
    }
}