use bytemuck::{Pod, Zeroable};

mod default_material;
pub use default_material::*;
mod curve2d_data;
pub use curve2d_data::*;
mod curve2d_renderer;
pub use curve2d_renderer::*;

use glam::*;

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub struct Curve2DSegment {
    start_index: usize,
    end_index: usize,
    direction: Vec2,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Curve2DVertexData {
    position: Vec2,
    direction: Vec2,
    uv: Vec2,
}

impl Curve2DVertexData {
    pub const ELEM_SIZE: u32 = (std::mem::size_of::<f32>() * 2) as u32;
    pub const SIZE: u32 = Self::ELEM_SIZE * 3;
}

pub enum CurveType {
    Separate,
    Connected,
    RoundCorner {
        //angle between round connectors
        min_angle: f32,
    },
}

pub enum Curve2DDirection {
    Inside,
    Center,
    Outside,
}

#[derive(Clone, Copy, PartialEq)]
pub enum UVLengthAlignedValue {
    Index,
    IndexNormalized,
    Length,
    LengthNormalized,
}
