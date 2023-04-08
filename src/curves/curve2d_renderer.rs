use std::rc::Rc;

use glam::*;
use rust_webgl2::*;

use crate::{console_log_format, curves::UVLengthAlignedValue, renderer::Renderer};

use super::{curve2d_data::Curve2DData, Curve2DVertexData, CurveType};

fn push_line_segment(
    position_direction: &mut Vec<Curve2DVertexData>,
    indices: &mut Vec<u32>,
    start: Vec2,
    end: Vec2,
    start_dir: (Vec2, Vec2),
    end_dir: (Vec2, Vec2),
) {
    let start_index = position_direction.len() as u32;

    let start_0 = Curve2DVertexData {
        position: start,
        direction: start_dir.0,
        uv: vec2(0.0, 1.0),
    };

    let start_1 = Curve2DVertexData {
        position: start,
        direction: start_dir.1,
        uv: vec2(0.0, -1.0),
    };

    position_direction.push(start_0);
    position_direction.push(start_1);

    let end_0 = Curve2DVertexData {
        position: end,
        direction: end_dir.0,
        uv: vec2(1.0, 1.0),
    };

    let end_1 = Curve2DVertexData {
        position: end,
        direction: end_dir.1,
        uv: vec2(1.0, -1.0),
    };

    position_direction.push(end_0);
    position_direction.push(end_1);

    let triangle = (0, 1, 2);
    indices.push(start_index + triangle.0);
    indices.push(start_index + triangle.1);
    indices.push(start_index + triangle.2);
    let triangle = (2, 1, 3);
    indices.push(start_index + triangle.0);
    indices.push(start_index + triangle.1);
    indices.push(start_index + triangle.2);
}

pub fn set_curve_2d_data_separate(
    position_direction: &mut Vec<Curve2DVertexData>,
    indices: &mut Vec<u32>,
    curve: &Curve2DData,
) {
    fn get_direction(curve: &Curve2DData, index: (usize, usize)) -> Vec2 {
        let start = curve.points[index.0];
        let end = curve.points[index.1];

        let edge_direction = (end - start).normalize();
        let edge_direction = vec2(edge_direction.y, -edge_direction.x);

        edge_direction
    }
    // generate line segments
    if curve.points.len() > 1 {
        let mut edge_index_directions: Vec<((usize, usize), Vec2)> = Vec::new();

        for i in 0..(curve.points.len() - 1) {
            let edge_indices = (i, i + 1);
            let edge_direction = get_direction(curve, edge_indices);

            edge_index_directions.push((edge_indices, edge_direction));
        }

        if curve.close {
            let edge_indices = (curve.points.len() - 1, 0);
            let edge_direction = get_direction(curve, edge_indices);

            edge_index_directions.push((edge_indices, edge_direction));
        }

        for edge in edge_index_directions {
            let start = curve.points[(edge.0).0];
            let end = curve.points[(edge.0).1];
            let dir = edge.1;

            let vert_dir = (dir, -dir);
            push_line_segment(position_direction, indices, start, end, vert_dir, vert_dir);
        }
    }
}

pub fn set_curve_2d_data_connected(
    position_direction: &mut Vec<Curve2DVertexData>,
    indices: &mut Vec<u32>,
    curve: &Curve2DData,
) {
    let start_index = position_direction.len();
    let max_count = if curve.close {
        curve.points.len()
    } else {
        curve.points.len() - 1
    };

    let curve_length = if let UVLengthAlignedValue::Length | UVLengthAlignedValue::LengthNormalized = curve.uv_x_kind {
        curve.get_curve_length()
    } else {
        0.0
    };

    let mut current_lenght = 0.0;

    for corner_index in 0..max_count + 1 {
        let uv_x = match curve.uv_x_kind {
            UVLengthAlignedValue::Index => corner_index as f32,
            UVLengthAlignedValue::IndexNormalized => corner_index as f32 / max_count as f32,
            UVLengthAlignedValue::Length | UVLengthAlignedValue::LengthNormalized => {
                let segment_length = if corner_index == 0 {
                    0.0
                } else {
                    curve.segment_length(corner_index - 1).unwrap()
                };
                current_lenght += segment_length;

                if curve.uv_x_kind == UVLengthAlignedValue::LengthNormalized {
                    current_lenght / curve_length
                } else {
                    current_lenght
                }
            }
        };

        let corner_index = corner_index % max_count;

        let direction = curve.get_corner_direction(corner_index).unwrap();
        let directions = curve.generate_directions(direction);

        let pos = curve.points[corner_index];
        let v_0 = Curve2DVertexData {
            position: pos,
            direction: directions.0,
            uv: vec2(uv_x, 1.0),
        };
        let v_1 = Curve2DVertexData {
            position: pos,
            direction: directions.1,
            uv: vec2(uv_x, -1.0),
        };
        position_direction.push(v_0);
        position_direction.push(v_1);
    }

    const TRI0: [usize; 3] = [0, 1, 2];
    const TRI1: [usize; 3] = [2, 1, 3];

    let segment_count = curve.last_segment_index() + 1;
    for seg_index in 0..segment_count {
        for t_i in TRI0 {
            let index = (seg_index * 2) + start_index + t_i;
            indices.push(index as u32)
        }
        for t_i in TRI1 {
            let index = (seg_index * 2) + start_index + t_i;
            indices.push(index as u32)
        }
    }
}

pub struct Curve2DRenderer {
    pub transform: Mat4,
    pub vertex_array_object: Option<Rc<GlVertexArrayObject>>,

    data_count: usize,
    pub index_type: IndexType,
    pub index_count: usize,
    pub render_index_count: usize,
}

impl Curve2DRenderer {
    pub fn new() -> Self {
        let transform = Mat4::IDENTITY;
        Self {
            transform,
            vertex_array_object: None,

            data_count: 0,
            index_count: 0,
            render_index_count: 0,
            index_type: IndexType::U32,
        }
    }

    pub fn set_line_data<'a>(
        &mut self,
        renderer: &Renderer,
        curves: &Vec<Curve2DData>,
        curve_type: CurveType,
    ) {
        let mut curve2d_vertex_data = Vec::<Curve2DVertexData>::new();
        let mut indices = Vec::<u32>::new();

        for curve in curves {
            match curve_type {
                CurveType::Separate => {
                    set_curve_2d_data_separate(&mut curve2d_vertex_data, &mut indices, curve)
                }
                CurveType::Connected => {
                    set_curve_2d_data_connected(&mut curve2d_vertex_data, &mut indices, curve)
                }
                _ => panic!("Impossible curve type"),
            }
        }

        self.data_count = curve2d_vertex_data.len();
        self.index_count = indices.len();
        self.render_index_count = indices.len();

        match &self.vertex_array_object {
            Some(vertex_array_object) => {
                if self.data_count >= curve2d_vertex_data.len() && self.index_count >= indices.len()
                {
                    Self::update_data_buffers(vertex_array_object, &curve2d_vertex_data, &indices);
                } else {
                    let (data_buffer, index_buffer) =
                        Self::create_data_buffers(renderer, &curve2d_vertex_data, &indices);
                    vertex_array_object.swap_buffer(renderer.get_graphics(), 0, data_buffer);
                    vertex_array_object
                        .swap_index_buffer(renderer.get_graphics(), Some(index_buffer));
                }
            }
            _ => {
                let (data_buffer, index_buffer) =
                    Self::create_data_buffers(renderer, &curve2d_vertex_data, &indices);
                self.vertex_array_object = Some(Self::create_vertex_array_object(
                    renderer,
                    &vec![&data_buffer],
                    index_buffer,
                ));
            }
        }
    }

    fn create_vertex_array_object(
        renderer: &Renderer,
        buffers: &[&Rc<GlBuffer>],
        index_buffer: Rc<GlIndexBuffer>,
    ) -> Rc<GlVertexArrayObject> {
        let stride = Curve2DVertexData::SIZE as u8;
        let attributes = vec![
            AttributeDescription {
                location: 0,
                unit_type: NumberType::FLOAT,
                size: AttributeSize::FOUR,
                buffer: 0,
                normalize: false,
                kind: AttributeType::Interleaved { stride, offset: 0 },
            },
            AttributeDescription {
                location: 1,
                unit_type: NumberType::FLOAT,
                size: AttributeSize::TWO,
                buffer: 0,
                normalize: false,
                kind: AttributeType::Interleaved {
                    stride,
                    offset: Curve2DVertexData::ELEM_SIZE * 2,
                },
            },
        ];
        Rc::new(
            GlVertexArrayObject::new(
                &renderer.get_graphics(),
                attributes,
                buffers,
                Some(index_buffer),
            )
            .unwrap(),
        )
    }

    fn update_data_buffers(
        vertex_array_object: &Rc<GlVertexArrayObject>,
        curve2d_vertex_data: &Vec<Curve2DVertexData>,
        indices: &Vec<u32>,
    ) {
        vertex_array_object.get_array_buffers()[0]
            .upgrade()
            .unwrap()
            .buffer_data(&curve2d_vertex_data);
        vertex_array_object
            .get_index_buffer()
            .unwrap()
            .upgrade()
            .unwrap()
            .buffer_data(&indices);
    }

    fn create_data_buffers(
        renderer: &Renderer,
        vertex_data: &Vec<Curve2DVertexData>,
        indices: &Vec<u32>,
    ) -> (Rc<GlBuffer>, Rc<GlIndexBuffer>) {
        console_log_format!("Curve buffers created");
        let data_buffer = Rc::new(
            GlBuffer::array_buffer_with_data(
                &renderer.get_graphics(),
                &vertex_data,
                BufferUsage::STATIC_DRAW,
            )
            .unwrap(),
        );

        let index_buffer = Rc::new(
            GlIndexBuffer::with_data(
                &renderer.get_graphics(),
                IndexType::U32,
                &indices,
                BufferUsage::STATIC_DRAW,
            )
            .unwrap(),
        );

        (data_buffer, index_buffer)
    }

    pub fn request_render(
        graphics: &Graphics,
        vao: &GlVertexArrayObject,
        index_count: u32,
        index_type: IndexType,
        material: &mut GlMaterial,
    ) {
        material.set_capabilities(graphics, 0);
        vao.bind();

        let mut current_program = material.program.use_program();
        current_program.push_all_uniforms();
        current_program.draw_elements_with_i32(
            PrimitiveType::TRIANGLES,
            index_count,
            index_type,
            0,
        );

        vao.unbind();
    }
}
