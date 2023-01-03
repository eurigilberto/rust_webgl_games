#![allow(dead_code)]
use crate::mesh::mesh_data::*;
use glam::*;
pub mod cube;
pub mod icosahaedron;

pub fn generate_rect(width: f32, height: f32) -> MeshData {
    let mut vertices: Vec<Vec3> = vec![Vec3::ZERO; 4];
    let mut uv: Vec<Vec2> = vec![Vec2::ZERO; 4];
    let triangles: Vec<[u16; 3]> = vec![[0, 2, 1], [2, 3, 1]];
    let half_size = vec2(width / 2.0, height / 2.0);

    vertices[0] = vec3(-half_size.x, 0.0, -half_size.y);
    uv[0] = vec2(0.0, 0.0);
    vertices[1] = vec3(-half_size.x, 0.0, half_size.y);
    uv[1] = vec2(0.0, 1.0);
    vertices[2] = vec3(half_size.x, 0.0, -half_size.y);
    uv[2] = vec2(1.0, 0.0);
    vertices[3] = vec3(half_size.x, 0.0, half_size.y);
    uv[3] = vec2(1.0, 1.0);

    let mut triangles_ = Vec::new();
    for tri in triangles {
        triangles_.push(tri[0]);
        triangles_.push(tri[1]);
        triangles_.push(tri[2]);
    }

    MeshData {
        positions: vertices,
        normals: None,
        uvs: Some(vec![uv]),
        indices: Some(IndexData::U16(triangles_)),
        colors: None,
    }
}

/// edge count is going to be clamped to 4
pub fn generate_circle(radius: f32, edge_count: u32) -> MeshData {
    let center = Vec3::ZERO;
    let edge_count = if edge_count < 4 { 4 } else { edge_count };
    let arc_angle = f32::to_radians(360.0 / (edge_count as f32));

    let mut positions = Vec::new();
    let mut indices = IndexData::U32(Vec::new());

    positions.push(center);
    for i in 0..edge_count {
        let i = i as f32;
        let arc_angle = i * arc_angle;
        let pos_2d = vec2(f32::cos(arc_angle), f32::sin(arc_angle)) * radius;
        let position = vec3(pos_2d.x, 0.0, pos_2d.y);
        positions.push(position);
    }

    let init_edge_pos: usize = 1;
    for i in 0..(edge_count as usize) {
        let edge_0 = init_edge_pos + i;
        let edge_1 = init_edge_pos + (i + 1) % (edge_count as usize);
        let triangle = [edge_1 as u32, edge_0 as u32, 0 as u32];

        for tri in triangle {
            indices.push_u32(tri);
        }
    }

    MeshData {
        positions,
        indices: Some(indices),
        normals: None,
        uvs: None,
        colors: None,
    }
}

pub fn generate_cilinder(
    edge_count: u32,
    radius: f32,
    height: f32,
    _length_segments: u32,
) -> MeshData {
    let ring_vert_count = edge_count + 1;
    let edge_count = if edge_count < 4 { 4 } else { edge_count };
    /*let length_segments = if length_segments < 2 {
        2
    } else {
        length_segments
    };*/
    let arc_angle = f32::to_radians(360.0 / (edge_count as f32));

    let mut positions = Vec::new();
    let mut indices = IndexData::U32(Vec::new());
    let mut uv = Vec::new();

    for i in 0..ring_vert_count {
        let pos_2d = {
            let i = i as f32;
            let arc_angle = i * arc_angle;
            vec2(f32::cos(arc_angle), f32::sin(arc_angle)) * radius
        };

        let uv_x = (i as f32) / (edge_count as f32);

        positions.push(vec3(pos_2d.x, 0.0, pos_2d.y));
        uv.push(vec2(uv_x, 0.0));
        positions.push(vec3(pos_2d.x, height, pos_2d.y));
        uv.push(vec2(uv_x, 1.0));
    }

    for i in 0..edge_count {
        let i = (i * 2) as u32;
        //let edge_count = edge_count as u32;

        let triangle = [i + 1, i + 2, i];
        indices.push_u32_triangle(triangle);
        let triangle = [i + 1, i + 3, i + 2];
        indices.push_u32_triangle(triangle);
    }

    MeshData {
        positions,
        normals: None,
        uvs: Some(vec![uv]),
        indices: Some(indices),
        colors: None,
    }
}

pub fn generate_ring(inner_radius: f32, outer_radius: f32, edge_count: u32) -> MeshData {
    let edge_count = if edge_count < 4 { 4 } else { edge_count };
    let arc_angle = f32::to_radians(360.0 / (edge_count as f32));

    let mut positions = Vec::new();
    let mut indices = IndexData::U32(Vec::new());

    let inner_start: usize = 0;
    for i in 0..edge_count {
        let i = i as f32;
        let arc_angle = i * arc_angle;
        let pos_2d = vec2(f32::cos(arc_angle), f32::sin(arc_angle)) * inner_radius;
        let position = vec3(pos_2d.x, 0.0, pos_2d.y);
        positions.push(position);
    }

    let outer_start = positions.len();
    for i in 0..edge_count {
        let i = i as f32;
        let arc_angle = i * arc_angle;
        let pos_2d = vec2(f32::cos(arc_angle), f32::sin(arc_angle)) * outer_radius;
        let position = vec3(pos_2d.x, 0.0, pos_2d.y);
        positions.push(position);
    }

    for i in 0..(edge_count as usize) {
        //inner_outer
        let offset_0 = i;
        let offset_1 = (i + 1) % (edge_count as usize);

        let inner_edge_0 = (inner_start + offset_0) as u32;
        let inner_edge_1 = (inner_start + offset_1) as u32;

        let outer_edge_0 = (outer_start + offset_0) as u32;
        let outer_edge_1 = (outer_start + offset_1) as u32;

        let triangle_inner = [outer_edge_0, inner_edge_0, inner_edge_1];
        let triangle_outer = [outer_edge_1, outer_edge_0, inner_edge_1];

        indices.push_u32_triangle(triangle_inner);
        indices.push_u32_triangle(triangle_outer);
    }

    MeshData {
        positions,
        indices: Some(indices),
        normals: None,
        uvs: None,
        colors: None,
    }
}

pub fn generate_arc(radius: f32, arc_angle_deg: f32, edge_count: u32) -> MeshData {
    let center = Vec3::ZERO;
    let edge_count: usize = if edge_count < 4 {
        4
    } else {
        edge_count as usize
    };
    let max_arc_angle_deg = arc_angle_deg.max(0.0).min(360.0);
    let arc_angle = f32::to_radians(max_arc_angle_deg / (edge_count as f32));

    let mut positions = Vec::new();
    let mut indices = IndexData::U32(Vec::new());

    positions.push(center);
    for i in 0..(edge_count + 1) {
        let i = i as f32;
        let arc_angle = i * arc_angle;
        let pos_2d = vec2(f32::cos(arc_angle), f32::sin(arc_angle)) * radius;
        let position = vec3(pos_2d.x, 0.0, pos_2d.y);
        positions.push(position);
    }

    let init_edge_pos: usize = 1;
    for i in 0..(edge_count as usize) {
        let edge_0 = init_edge_pos + i;
        let edge_1 = init_edge_pos + (i + 1);
        let triangle = [0, edge_0 as u32, edge_1 as u32];
        indices.push_u32_triangle(triangle);
    }

    MeshData {
        positions,
        indices: Some(indices),
        normals: None,
        uvs: None,
        colors: None,
    }
}

pub fn generate_cone(
    radius: f32,
    height: f32,
    segment_count: usize,
    reverse_height: Option<f32>,
) -> (Vec<Vec3>, Vec<[usize; 3]>) {
    let top_vertex = vec3(0.0, height, 0.0);
    let mut positions = Vec::new();
    positions.push(top_vertex);

    let segment_count = if segment_count < 3 { 3 } else { segment_count };
    let segment_angle = f32::to_radians(360.0 / (segment_count as f32));
    for i in 0..segment_count {
        let fi = i as f32;
        let pos = vec2(f32::cos(fi * segment_angle), f32::sin(fi * segment_angle)) * radius;
        positions.push(vec3(pos.x, 0.0, pos.y));
    }

    const TOP_VERTEX: usize = 0;
    const RING_START: usize = 1;

    let mut ring_edges = Vec::new();
    for idx in 0..segment_count {
        ring_edges.push([idx + RING_START, ((idx + 1) % segment_count) + RING_START]);
    }

    let mut triangles = vec![];
    //Main triangles
    for edge in ring_edges.iter() {
        triangles.push([TOP_VERTEX, edge[1], edge[0]]);
    }

    match reverse_height {
        Some(reverse_height) => {
            positions.push(vec3(0.0, -reverse_height, 0.0));
            let bot_vertex = positions.len() - 1;
            for edge in ring_edges.iter() {
                triangles.push([edge[0], edge[1], bot_vertex]);
            }
        }
        None => {
            //BottomCap
            for idx in 1..segment_count - 1 {
                let edge = [idx + RING_START, idx + 1 + RING_START];
                triangles.push([RING_START, edge[0], edge[1]]);
            }
        }
    }
    return (positions, triangles);
}

pub fn generate_cone_flat(
    radius: f32,
    height: f32,
    segment_count: usize,
    reverse_height: Option<f32>,
) -> MeshData {
    let (position, triangles) = generate_cone(radius, height, segment_count, reverse_height);
    let mut m_positions = Vec::new();
    let mut m_normals = Vec::new();
    for tri in triangles.iter() {
        let tri_positions = [
            position[tri[0]],
            position[tri[1]],
            position[tri[2]],
        ];
        let normal = get_triangle_normal(tri_positions);
        for position in tri_positions.iter(){
            m_normals.push(normal);
            m_positions.push(*position);
        }
    }
    MeshData {
        positions: m_positions,
        normals: Some(m_normals),
        uvs: None,
        colors: None,
        indices: None,
    }
}

pub fn get_triangle_normal(triangle: [Vec3; 3]) -> Vec3 {
    let dir_0 = triangle[0] - triangle[1];
    let dir_1 = triangle[2] - triangle[1];
    (dir_0.cross(dir_1)).normalize()
}
