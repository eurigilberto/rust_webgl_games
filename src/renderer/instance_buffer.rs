use std::{rc::Rc, mem::size_of};

use rust_webgl2::{GlBuffer, BindingPoint, BufferUsage};

use super::Renderer;

pub struct InstanceBuffer {
    pub buffers: Vec<Rc<GlBuffer>>,
}

impl InstanceBuffer {
    pub fn new(attributes_count: usize, instance_count: usize, renderer: &Renderer) -> Self {
        let mut buffers = Vec::new();
        for _ in 0..attributes_count {
            let u32_byte_size = size_of::<u32>() as u32;
            let vec4_u32_byte_size = u32_byte_size * 4;
            buffers.push(Rc::new(
                GlBuffer::with_size(
                    renderer.get_graphics(),
                    BindingPoint::ARRAY_BUFFER,
                    vec4_u32_byte_size * instance_count as u32,
                    BufferUsage::DYNAMIC_DRAW,
                )
                .expect("Coudl not create GLBuffer"),
            ));
        }
        Self { buffers }
    }
}