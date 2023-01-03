use glam::*;
use rust_webgl2::*;

pub struct SimpleIcoMesh {
    pub position_buffer: GlBuffer,
    pub normal_buffer: GlBuffer,
    pub vertex_count: u32,
}

pub fn subdivide_triangle(triangle: (Vec3, Vec3, Vec3)) -> Vec<(Vec3, Vec3, Vec3)> {
    let edge0 = (triangle.0, triangle.1);
    let edge1 = (triangle.1, triangle.2);
    let edge2 = (triangle.2, triangle.0);

    let half_edge0 = Vec3::lerp(edge0.0, edge0.1, 0.5);
    let half_edge1 = Vec3::lerp(edge1.0, edge1.1, 0.5);
    let half_edge2 = Vec3::lerp(edge2.0, edge2.1, 0.5);

    let mut triangles = Vec::new();
    triangles.push((triangle.0, half_edge0, half_edge2));

    triangles.push((half_edge0, triangle.1, half_edge1));

    triangles.push((half_edge2, half_edge1, triangle.2));

    triangles.push((half_edge0, half_edge1, half_edge2));

    triangles
}

impl SimpleIcoMesh {
    pub fn new(graphics: &Graphics, edge_length: f32) -> Result<SimpleIcoMesh, ()> {
        let ico_mesh = icosahaedron::get_positions(edge_length);
        let mut position_data = Vec::new();
        let mut normal_data = Vec::new();
        let mut vertex_count = 0;

        let index_data = ico_mesh.indices.as_ref().unwrap();
		for triangle in index_data.get_iter_triangle(){
			let pos = &ico_mesh.positions;
            let tri_positions = (
                pos[triangle[0]],
                pos[triangle[1]],
                pos[triangle[2]],
            );
            let normal =
                Vec3::normalize((tri_positions.0 + tri_positions.1 + tri_positions.2) / 3.0);

            position_data.push(tri_positions.0);
            normal_data.push(normal);

            position_data.push(tri_positions.1);
            normal_data.push(normal);

            position_data.push(tri_positions.2);
            normal_data.push(normal);

            vertex_count += 3;
		}

        let position_buffer =
            GlBuffer::array_buffer_with_data(graphics, &position_data, BufferUsage::STATIC_DRAW);
        let normal_buffer =
            GlBuffer::array_buffer_with_data(graphics, &normal_data, BufferUsage::STATIC_DRAW);

        match (position_buffer, normal_buffer) {
            (Ok(position_buffer), Ok(normal_buffer)) => Ok(SimpleIcoMesh {
                position_buffer,
                normal_buffer,
                vertex_count,
            }),
            _ => Err(()),
        }
    }

    pub fn with_subdivision(
        graphics: &Graphics,
        edge_length: f32,
        subdiv: u32,
    ) -> Result<SimpleIcoMesh, ()> {
        let ico_mesh = icosahaedron::get_positions(edge_length);
        let mut position_data = Vec::new();
        let mut normal_data = Vec::new();
        let mut vertex_count = 0;

        let radius = icosahaedron::get_circumscribed_sphere_radius(edge_length);

        let index_data = ico_mesh.indices.as_ref().unwrap();
        for triangle in index_data.get_iter_triangle(){
            let pos = &ico_mesh.positions;
            let triangle = (
                pos[triangle[0] as usize],
                pos[triangle[1] as usize],
                pos[triangle[2] as usize],
            );
            //let normal = Vec3::normalize((triangle.0 + triangle.1 + triangle.2) / 3.0);

            let mut triangle_queue = Vec::new();
            triangle_queue.push(triangle);
            for _ in 0..subdiv {
                let mut extra_triangles = Vec::new();
                for tri in triangle_queue.drain(..) {
                    let mut subdiv_triangle = subdivide_triangle(tri);
                    for s_tri in subdiv_triangle.drain(..) {
                        extra_triangles.push(s_tri);
                    }
                }
                triangle_queue.extend(extra_triangles);
            }

            for triangle in triangle_queue {
                let mut triangle = triangle;

                triangle.0 = triangle.0.normalize() * radius;
                triangle.1 = triangle.1.normalize() * radius;
                triangle.2 = triangle.2.normalize() * radius;

                let n_normal = Vec3::normalize((triangle.0 + triangle.1 + triangle.2) / 3.0);

                let normal = n_normal; // (normal * 0.5 + n_normal * 0.5).normalize();

                position_data.push(triangle.0);
                normal_data.push(normal);

                position_data.push(triangle.1);
                normal_data.push(normal);

                position_data.push(triangle.2);
                normal_data.push(normal);

                vertex_count += 3;
            }
        }

        let position_buffer =
            GlBuffer::array_buffer_with_data(graphics, &position_data, BufferUsage::STATIC_DRAW);
        let normal_buffer =
            GlBuffer::array_buffer_with_data(graphics, &normal_data, BufferUsage::STATIC_DRAW);

        match (position_buffer, normal_buffer) {
            (Ok(position_buffer), Ok(normal_buffer)) => Ok(SimpleIcoMesh {
                position_buffer,
                normal_buffer,
                vertex_count,
            }),
            _ => Err(()),
        }
    }
}

mod icosahaedron {
    use crate::mesh::mesh_data::{IndexData, MeshData};

    use glam::*;

    pub fn get_circumscribed_sphere_radius(edge_length: f32) -> f32 {
        let multiplier = f32::sqrt(10.0 + 2.0 * f32::sqrt(5.0)) / 4.0;
        edge_length * multiplier
    }
    pub fn get_pentagon_circle_radius(edge_length: f32) -> f32 {
        let radius_multiplier = f32::sqrt((5.0 - f32::sqrt(5.0)) / 2.0);
        let radius = edge_length / radius_multiplier;
        radius
    }
    pub fn get_pentagon_height(edge_length: f32) -> f32 {
        let radius = get_pentagon_circle_radius(edge_length);
        let radius_2 = radius * radius;
        let edge_len_2 = edge_length * edge_length;

        let height = f32::sqrt(edge_len_2 - radius_2);
        height
    }

    pub fn get_pentagon_points(edge_length: f32, angle_offset: f32) -> [Vec2; 5] {
        let pentagon_radius = get_pentagon_circle_radius(edge_length);
        let pentagon_section_angle = 360.0 / 5.0;
        let section_angle_rad = f32::to_radians(pentagon_section_angle);

        let mut points: [Vec2; 5] = [vec2(0.0, 0.0); 5];
        for (index, point) in points.iter_mut().enumerate() {
            let angle = (index as f32) * section_angle_rad + angle_offset;
            *point = vec2(
                f32::cos(angle) * pentagon_radius,
                f32::sin(angle) * pentagon_radius,
            );
        }
        points
    }

    pub fn get_positions(edge_length: f32) -> MeshData {
        let sphere_radius = get_circumscribed_sphere_radius(edge_length);
        let center = vec3(0.0, sphere_radius, 0.0);
        let pentagon_height = sphere_radius - get_pentagon_height(edge_length);

        let top_pentagon_points = get_pentagon_points(edge_length, 0.0);
        let bottom_offset = f32::to_radians((360.0 / 5.0) * 0.5);
        let bottom_pentagon_points = get_pentagon_points(edge_length, bottom_offset);

        let mut positions = Vec::new();
        let mut indices = IndexData::U32(Vec::new());

        let top_center_index = 0;
        positions.push(center);
        for point in top_pentagon_points {
            positions.push(vec3(point.x, pentagon_height, point.y));
        }

        let bot_center_index = positions.len();
        positions.push(-1.0 * center);
        for point in bottom_pentagon_points {
            positions.push(vec3(point.x, -1.0 * pentagon_height, point.y));
        }

        for index in 0..5 {
            let init_index = 1;

            let id0 = top_center_index;
            let id1 = init_index + index;
            let id2 = init_index + ((index + 1) % 5);
            let triangle = [id2 as u32, id1 as u32, id0 as u32];
            indices.push_u32_triangle(triangle);
        }

        for index in 0..5 {
            let init_index = bot_center_index + 1;
            let edge_0 = (index + 1) % 5;

            let id0 = init_index + edge_0;
            let id1 = init_index + index;
            let id2 = bot_center_index;
            let triangle = [id2 as u32, id1 as u32, id0 as u32];
            indices.push_u32_triangle(triangle);
        }

        for index in 0..10 {
            let id = index % 2;
            let edge_id = index >> 1;

            let top_init_index = 1;
            let bot_init_index = bot_center_index + 1;

            let e0 = edge_id;
            let e1 = (edge_id + 1) % 5;

            if id == 0 {
                let triangle = [
                    (top_init_index + e0) as u32,
                    (top_init_index + e1) as u32,
                    (bot_init_index + e0) as u32,
                ];
                indices.push_u32_triangle(triangle);
            } else if id == 1 {
                let triangle = [
                    (top_init_index + e1) as u32,
                    (bot_init_index + e1) as u32,
                    (bot_init_index + e0) as u32,
                ];
                indices.push_u32_triangle(triangle);
            }
        }

        MeshData {
            positions,
            indices: Some(indices),
            normals: None,
            uvs: None,
			colors: None
        }
    }
}