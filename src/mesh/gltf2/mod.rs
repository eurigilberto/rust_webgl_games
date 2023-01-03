use std::rc::Rc;

use glam::*;
use gltf::mesh::Reader;
use rust_webgl2::*;

pub mod material;
mod shader;

use super::mesh_data::{IndexData, MeshBuffers, MeshData};

pub fn generate_all_mesh_from_gltf<'a, 'err>(
    file_buffer: &[u8],
) -> Result<Vec<Gltf2Mesh>, &'err str> {
    let glb_file = file_buffer.to_vec();
    let mut mesh_index = 0;
    match gltf::import_slice(glb_file.as_slice()) {
        Ok((document, buffer_vec, _image_vec)) => {
            let mut gltf_mesh_vec = Vec::new();
            for mesh in document.meshes() {
                let mesh_name = match mesh.name() {
                    Some(name) => Some(String::from(name)),
                    None => None,
                };
                match generate_mesh_from_primitives(
                    &mesh,
                    &buffer_vec,
                    mesh_name,
                    &mut mesh_index,
                ) {
                    Ok(mesh_vec) => gltf_mesh_vec.extend(mesh_vec),
                    Err(_) => return Err("Gltf2 mesh could not be generated"),
                }
            }
            Ok(gltf_mesh_vec)
        }
        Err(_) => return Err("Gltf2 mesh could not be generated"),
    }
}

pub fn generate_mesh_from_primitives(
    mesh_data: &gltf::Mesh,
    buffer_vec: &Vec<gltf::buffer::Data>,
    mesh_name: Option<String>,
    mesh_index: &mut u32,
) -> Result<Vec<Gltf2Mesh>, ()> {
    let mut gltf_mesh_vec = Vec::new();
    for primitive in mesh_data.primitives() {
        let mesh_name = match &mesh_name {
            Some(m_name) => {
                let mut mesh_name = m_name.clone();
                let name_padding = format!("_mesh_{}", mesh_index);
                mesh_name.push_str(name_padding.as_str());
                Some(mesh_name)
            }
            None => None,
        };
        let reader = primitive.reader(|buffer| Some(&buffer_vec[buffer.index()]));
        match Gltf2Mesh::from_reader( reader, mesh_name) {
            Ok(gltf_2_mesh) => {
                gltf_mesh_vec.push(gltf_2_mesh);
            }
            Err(_) => return Err(()),
        }
        *mesh_index += 1;
    }
    Ok(gltf_mesh_vec)
}

macro_rules! iter_indices {
    ($idx_iter: ident, $indices: ident) => {
        for idx in $idx_iter {
            $indices.push(idx)
        }
    };
}

pub fn generate_gltf2_mesh_data<'a, 's, 'err, F>(
    reader: Reader<'a, 's, F>,
) -> Result<MeshData, &'err str>
where
    F: Clone + Fn(gltf::Buffer<'a>) -> Option<&'s [u8]>,
{
    let vertex_positions = {
        let mut positions = Vec::new();
        match reader.read_positions() {
            Some(position_iter) => {
                for p in position_iter {
                    positions.push(glam::vec3(p[0], p[1], p[2]));
                }
                positions
            }
            None => return Err("No vertex position"),
        }
    };

    let opt_vertex_normals = {
        let mut normals = Vec::new();
        match reader.read_normals() {
            Some(normal_iter) => {
                for p in normal_iter {
                    normals.push(glam::vec3(p[0], p[1], p[2]));
                }
                Some(normals)
            }
            None => None,
        }
    };

    let vertex_uvs = {
        let mut uvs = Vec::new();
        for i in 0..12 {
            let mut uv_i = Vec::new();
            if let Some(uv_iter) = reader.read_tex_coords(i) {
                for uv in uv_iter.into_f32() {
                    uv_i.push(vec2(uv[0], uv[1]));
                }
                uvs.push(uv_i);
            }
        }
        uvs
    };

    let mut vertex_colors = Vec::new();
    for i in 0..12 {
        if let Some(color_iter) = reader.read_colors(i) {
            let mut colors_i = Vec::new();
            for _c in color_iter.into_rgba_f32() {
                colors_i.push(RGBA::new(_c[0], _c[1], _c[2], _c[3]));
            }
            vertex_colors.push(colors_i);
        }
    }

    let opt_index_data = if let Some(index_reader) = reader.read_indices() {
        match index_reader {
            gltf::mesh::util::ReadIndices::U8(i_iter) => {
                let mut indices = Vec::new();
                iter_indices!(i_iter, indices);
                Some(IndexData::U8(indices))
            }
            gltf::mesh::util::ReadIndices::U16(i_iter) => {
                let mut indices = Vec::new();
                iter_indices!(i_iter, indices);
                Some(IndexData::U16(indices))
            }
            gltf::mesh::util::ReadIndices::U32(i_iter) => {
                let mut indices = Vec::new();
                iter_indices!(i_iter, indices);
                Some(IndexData::U32(indices))
            }
        }
    } else {
        None
    };

    return Ok(MeshData {
        positions: vertex_positions,
        normals: opt_vertex_normals,
        uvs: if vertex_uvs.len() > 0 {
            Some(vertex_uvs)
        } else {
            None
        },
        indices: opt_index_data,
        colors: None,
    });
}

pub fn create_mesh_buffers(graphics: &Graphics, gltf2_data: &MeshData) -> Result<MeshBuffers, ()> {
    let position_buffer = GlBuffer::with_data_static_array_buffer(graphics, &gltf2_data.positions)?;
    let mut vertex_buffers = Vec::new();
    vertex_buffers.push(Rc::new(position_buffer));

    if let Some(normals) = &gltf2_data.normals {
        vertex_buffers.push(Rc::new(GlBuffer::with_data_static_array_buffer(
            graphics, &normals,
        )?))
    }
    if let Some(uvs) = gltf2_data.uvs.as_ref() {
        for uv in uvs.iter() {
            vertex_buffers.push(Rc::new(GlBuffer::with_data_static_array_buffer(
                graphics, uv,
            )?))
        }
    }
    let index_buffer = match &gltf2_data.indices {
        Some(indices) => match indices {
            IndexData::U8(array) => Some(GlIndexBuffer::with_data(
                graphics,
                IndexType::U8,
                &array,
                BufferUsage::STATIC_DRAW,
            )?),
            IndexData::U16(array) => Some(GlIndexBuffer::with_data(
                graphics,
                IndexType::U16,
                &array,
                BufferUsage::STATIC_DRAW,
            )?),
            IndexData::U32(array) => Some(GlIndexBuffer::with_data(
                graphics,
                IndexType::U32,
                &array,
                BufferUsage::STATIC_DRAW,
            )?),
        },
        None => None,
    };

    Ok(MeshBuffers {
        vertex_buffers: vertex_buffers,
        index_buffer: match index_buffer {
            Some(index_buffer) => Some(Rc::new(index_buffer)),
            None => None,
        },
        index_count: match gltf2_data.indices.as_ref() {
            Some(indices) => Some(indices.index_count() as u32),
            None => None,
        },
    })
}

pub struct Gltf2Mesh {
    pub name: Option<String>,
    pub vertex_data: MeshData,
}

impl Gltf2Mesh {
    pub fn from_reader<'a, 's, F>(
        reader: Reader<'a, 's, F>,
        mesh_name: Option<String>,
    ) -> Result<Self, ()>
    where
        F: Clone + Fn(gltf::Buffer<'a>) -> Option<&'s [u8]>,
    {
        match generate_gltf2_mesh_data(reader) {
            Ok(gltf2_data) => Ok(Self {
                name: mesh_name,
                vertex_data: gltf2_data,
            }),
            Err(_) => Err(()),
        }
    }
}
