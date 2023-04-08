use std::rc::Rc;

use glam::*;
use rust_webgl2::*;

macro_rules! get_triangle_from_indices {
    ($indices: ident, $array: ident) => {
        [
            $array[$indices[0]] as usize,
            $array[$indices[1]] as usize,
            $array[$indices[2]] as usize,
        ]
    };
}

pub fn get_indices_data(data: &IndexData, indices: [usize; 3]) -> [usize; 3] {
    match data {
        IndexData::U8(array) => get_triangle_from_indices!(indices, array),
        IndexData::U16(array) => get_triangle_from_indices!(indices, array),
        IndexData::U32(array) => get_triangle_from_indices!(indices, array),
    }
}

pub fn get_triangle_strip_indices(data: &IndexData, index: usize) -> Option<[usize; 3]> {
    if index + 2 >= data.index_count() {
        return None;
    }
    let indices = if index % 2 == 0 {
        [index, index + 1, index + 2]
    } else {
        [index, index + 2, index + 1]
    };
    Some(get_indices_data(data, indices))
}
pub fn get_triangle_indices(data: &IndexData, index: usize) -> Option<[usize; 3]> {
    let tri_index = index * 3;
    if tri_index + 2 >= data.index_count() {
        return None;
    }
    let indices = [tri_index, tri_index + 1, tri_index + 2];
    Some(get_indices_data(data, indices))
}
pub fn get_triangle_fan_indices(data: &IndexData, index: usize) -> Option<[usize; 3]> {
    if index + 2 >= data.index_count() {
        return None;
    }
    let indices = [0, index + 1, index + 2];
    Some(get_indices_data(data, indices))
}

pub enum TriangleIteratorKind {
    Triangle,
    TriangleStrip,
    TriangleFan,
}

impl TriangleIteratorKind {
    pub fn at_index(&self, data: &IndexData, index: usize) -> Option<[usize; 3]> {
        match self {
            TriangleIteratorKind::Triangle => get_triangle_indices(data, index),
            TriangleIteratorKind::TriangleStrip => get_triangle_strip_indices(data, index),
            TriangleIteratorKind::TriangleFan => get_triangle_fan_indices(data, index),
        }
    }
}

pub struct IndexDataIter<'data> {
    data: &'data IndexData,
    index: usize,
    iterator: TriangleIteratorKind,
}

impl<'data> IndexDataIter<'data> {
    pub fn new(data: &'data IndexData, iterator: TriangleIteratorKind) -> Self {
        Self {
            data,
            index: 0,
            iterator,
        }
    }
}

impl<'data> Iterator for IndexDataIter<'data> {
    type Item = [usize; 3];

    fn next(&mut self) -> Option<Self::Item> {
        let data = self.iterator.at_index(self.data, self.index);
        self.index += 1;
        data
    }
}

#[derive(Debug)]
pub enum IndexData {
    U8(Vec<u8>),
    U16(Vec<u16>),
    U32(Vec<u32>),
}

impl IndexData {
    pub fn create_index_buffer(
        &self,
        graphics: &Graphics,
        usage: BufferUsage,
    ) -> Result<GlIndexBuffer, ()> {
        match self {
            IndexData::U8(data) => GlIndexBuffer::with_data(graphics, IndexType::U8, data, usage),
            IndexData::U16(data) => GlIndexBuffer::with_data(graphics, IndexType::U16, data, usage),
            IndexData::U32(data) => GlIndexBuffer::with_data(graphics, IndexType::U32, data, usage),
        }
    }

    pub fn index_count(&self) -> usize {
        match self {
            IndexData::U8(value) => value.len(),
            IndexData::U16(value) => value.len(),
            IndexData::U32(value) => value.len(),
        }
    }

    pub fn index_type(&self) -> IndexType {
        match self {
            IndexData::U8(_) => IndexType::U8,
            IndexData::U16(_) => IndexType::U16,
            IndexData::U32(_) => IndexType::U32,
        }
    }

    pub fn push_u8(&mut self, index: u8) {
        match self {
            IndexData::U8(indices) => indices.push(index),
            _ => {}
        }
    }
    pub fn push_u16_triangle(&mut self, triangle: [u16; 3]) {
        self.push_u16(triangle[0]);
        self.push_u16(triangle[1]);
        self.push_u16(triangle[2]);
    }
    pub fn push_u16(&mut self, index: u16) {
        match self {
            IndexData::U16(indices) => indices.push(index),
            _ => {}
        }
    }
    pub fn push_u32_triangle(&mut self, triangle: [u32; 3]) {
        self.push_u32(triangle[0]);
        self.push_u32(triangle[1]);
        self.push_u32(triangle[2]);
    }
    pub fn push_u32(&mut self, index: u32) {
        match self {
            IndexData::U32(indices) => indices.push(index),
            _ => {}
        }
    }

    pub fn get_iter_triangle(&self)->IndexDataIter{
        IndexDataIter::new(self, TriangleIteratorKind::Triangle)
    }
    pub fn get_iter_triangle_strip(&self)->IndexDataIter{
        IndexDataIter::new(self, TriangleIteratorKind::TriangleStrip)
    }
    pub fn get_iter_triangle_fan(&self)->IndexDataIter{
        IndexDataIter::new(self, TriangleIteratorKind::TriangleFan)
    }
}

pub struct MeshData {
    pub positions: Vec<Vec3>,
    pub normals: Option<Vec<Vec3>>,
    pub uvs: Option<Vec<Vec<Vec2>>>,
    pub colors: Option<Vec<Vec<RGBA>>>,
    pub indices: Option<IndexData>,
}

pub struct AttributeLocations {
    pub position_loc: u32,
    pub normals_loc: Option<u32>,
    pub uvs: Option<Vec<(usize, u32)>>,
}

pub struct MeshBuffers {
    pub vertex_buffers: Vec<Rc<GlBuffer>>,
    pub index_buffer: Option<Rc<GlIndexBuffer>>,
    pub index_count: Option<u32>,
}

impl MeshData {
    pub fn get_index_count(&self) -> usize {
        match &self.indices {
            Some(index_data) => index_data.index_count(),
            None => 0,
        }
    }
    pub fn generate_interleaved_vertex_array_object(
        &self,
        graphics: &Graphics,
        attribute_locations: AttributeLocations,
    ) -> Result<GlVertexArrayObject, String> {
        let mut stride = 0;
        let mut interleave_data: Vec<(u32, u32, AttributeSize)> = Vec::new();

        let vertex_count = self.positions.len();
        interleave_data.push((
            attribute_locations.position_loc,
            stride,
            AttributeSize::THREE,
        ));
        stride += 3;

        if let Some(normal_loc) = attribute_locations.normals_loc {
            interleave_data.push((normal_loc, stride, AttributeSize::THREE));
            stride += 3;
            if vertex_count != self.normals.as_ref().unwrap().len() {
                return Err("Normal does not have the correct count".into());
            }
        }
        if let Some(uv_locs) = &attribute_locations.uvs {
            for (uv_index, uv_loc) in uv_locs {
                interleave_data.push((*uv_loc, stride, AttributeSize::TWO));
                stride += 2;
                if vertex_count != self.uvs.as_ref().unwrap()[*uv_index].len() {
                    return Err(format!("Uv does not have the correct count {}", *uv_index));
                }
            }
        }
        let mut interleaved_vertex_buffer = Vec::new();
        for index in 0..vertex_count {
            let position = self.positions[index];
            interleaved_vertex_buffer.push(position.x);
            interleaved_vertex_buffer.push(position.y);
            interleaved_vertex_buffer.push(position.z);

            if let Some(_) = attribute_locations.normals_loc {
                let normal = self.normals.as_ref().unwrap()[index];
                interleaved_vertex_buffer.push(normal.x);
                interleaved_vertex_buffer.push(normal.y);
                interleaved_vertex_buffer.push(normal.z);
            }

            if let Some(uv_locs) = &attribute_locations.uvs {
                for (uv_index, _) in uv_locs {
                    let uv = self.uvs.as_ref().unwrap()[*uv_index][index];
                    interleaved_vertex_buffer.push(uv.x);
                    interleaved_vertex_buffer.push(uv.y);
                }
            }
        }

        let vertex_buffer = GlBuffer::array_buffer_with_data(
            graphics,
            &interleaved_vertex_buffer,
            BufferUsage::STATIC_DRAW,
        );
        if vertex_buffer.is_err() {
            return Err("Could not generate the GLBuffer".into());
        }
        let vertex_buffer = Rc::new(vertex_buffer.unwrap());

        let index_buffer = match &self.indices {
            Some(indices) => {
                let index_buffer = indices.create_index_buffer(graphics, BufferUsage::STATIC_DRAW);
                if index_buffer.is_err() {
                    return Err("Error creating index buffer".into());
                }
                Some(Rc::new(index_buffer.unwrap()))
            }
            None => None,
        };

        let mut attribute_descriptions = Vec::new();
        for (loc, offset, size) in interleave_data {
            let stride = stride as u8 * std::mem::size_of::<f32>() as u8;
            let offset = offset * std::mem::size_of::<f32>() as u32;
            attribute_descriptions.push(AttributeDescription {
                location: loc,
                unit_type: NumberType::FLOAT,
                size,
                buffer: 0,
                normalize: false,
                kind: AttributeType::Interleaved { stride, offset },
            });
        }

        let vertex_array_object = GlVertexArrayObject::new(
            graphics,
            attribute_descriptions,
            &vec![&vertex_buffer],
            match &index_buffer {
                Some(index_buffer) => Some(Rc::clone(index_buffer)),
                None => None,
            },
        );
        if vertex_array_object.is_err() {
            return Err("Error creating vertex array object".into());
        }
        let vertex_array_object = vertex_array_object.unwrap();

        Ok(vertex_array_object)
    }
}
