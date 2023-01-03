use glam::*;

pub mod cube{
    /*
    false false
    true  false
    true  true
    false true
     */

    use glam::{Vec3, vec3};

    pub const POSITIONS: [Vec3; 8] = [
        vec3(-0.5,  0.5, -0.5), //0
        vec3( 0.5,  0.5, -0.5), //1
        vec3( 0.5,  0.5,  0.5), //2
        vec3(-0.5,  0.5,  0.5), //3
        //----
        vec3(-0.5, -0.5, -0.5), //4
        vec3( 0.5, -0.5, -0.5), //5
        vec3( 0.5, -0.5,  0.5), //6
        vec3(-0.5, -0.5,  0.5), //7
    ];

    pub const QUAD_INDICES: [[usize; 4];6] = [
        [0, 1, 2, 3],// Y
        [7, 6, 5, 4],//-Y
        //----------
        [3, 2, 6, 7],// Z
        [4, 5, 1, 0],//-Z
        //----------
        [6, 2, 1, 5],// X
        [4, 0, 3, 7],//-X
    ];

    pub const TRI_INDICES: [[usize; 3]; 2] = [
        [0, 3, 2],
        [0, 2, 1],
    ];
}

pub fn rectangular_cuboid_smooth(
    width: f32,
    height: f32,
    depth: f32,
) -> (Vec<Vec3>, Vec<Vec3>, Vec<[u16; 3]>) {

    fn generate_indices(quad_idx: [usize; 4], index: usize)->[u16; 3]{
        let mut indices = [0; 3];
        for (index, idx) in cube::TRI_INDICES[index].into_iter().enumerate(){
            indices[index] = quad_idx[idx] as u16;
        }
        indices
    }

    let mut normals = Vec::new();
    for vert in cube::POSITIONS {
        normals.push(vert.normalize())
    }

    let mut vertices = Vec::new();
    for vert in cube::POSITIONS{
        vertices.push(vert * vec3(width, height, depth))
    }

    let mut indices = Vec::new();
    for i in 0..6{
        indices.push(generate_indices(cube::QUAD_INDICES[i], 0));
        indices.push(generate_indices(cube::QUAD_INDICES[i], 1));
    }

    (vertices, normals, indices)
}

pub fn rectangular_cuboid_flat(
    width: f32,
    height: f32,
    depth: f32,
) -> (Vec<Vec3>, Vec<Vec3>, Vec<[u16; 3]>) {
    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();
    
    for (index, quad_idx) in cube::QUAD_INDICES.into_iter().enumerate(){
        let start_offset = index * 4;
        for idx in quad_idx{
            vertices.push(cube::POSITIONS[idx] * vec3(width, height, depth));
        }

        let tri_idx = cube::TRI_INDICES[0];
        let origin_idx = quad_idx[tri_idx[1]];
        let a_idx = quad_idx[tri_idx[0]];
        let b_idx = quad_idx[tri_idx[2]];

        let dir_a = (cube::POSITIONS[a_idx] - cube::POSITIONS[origin_idx]).normalize();
        let dir_b = (cube::POSITIONS[b_idx] - cube::POSITIONS[origin_idx]).normalize();

        let normal = Vec3::cross(dir_a, dir_b);
        for _ in 0..4{
            normals.push(normal);
        }

        for tri in cube::TRI_INDICES{
            let mut triangle = [0; 3];
            for i in 0..3{
                triangle[i] = (tri[i] + start_offset) as u16;
            }
            indices.push(triangle);
        }
    }

    (vertices, normals, indices)
}