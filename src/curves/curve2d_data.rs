use glam::*;

use super::{Curve2DDirection, UVLengthAlignedValue};

pub struct Curve2DData {
    pub points: Vec<Vec2>,
    pub close: bool,
    pub expansion_direction: Curve2DDirection,
    pub invert_uv_direction: BVec2,
    pub uv_x_kind: UVLengthAlignedValue,
}

impl Curve2DData {
    pub fn new(points: Vec<Vec2>, close: bool) -> Self {
        Self {
            points,
            close,
            expansion_direction: Curve2DDirection::Center,
            invert_uv_direction: BVec2::new(false, false),
            uv_x_kind: UVLengthAlignedValue::Index,
        }
    }
    pub fn generate_directions(&self, direction: Vec2) -> (Vec2, Vec2) {
        match self.expansion_direction {
            Curve2DDirection::Inside => (Vec2::ZERO, -direction * 2.0),
            Curve2DDirection::Center => (direction, -direction),
            Curve2DDirection::Outside => (direction * 2.0, Vec2::ZERO),
        }
    }

    pub fn last_segment_index(&self) -> usize {
        if self.close {
            self.points.len() - 1
        } else {
            self.points.len() - 2
        }
    }

    pub fn get_segment_point_indices(&self, index: usize) -> Option<(usize, usize)> {
        if index == self.points.len() - 1 {
            if self.close {
                return Some((index, 0));
            } else {
                return None;
            }
        } else if index < self.points.len() - 1 {
            return Some((index, index + 1));
        } else {
            return None;
        };
    }

    pub fn segment_length(&self, index: usize) -> Option<f32> {
        let indices = self.get_segment_point_indices(index);
        let indices = if indices.is_none() {
            return None;
        } else {
            indices.unwrap()
        };

        Some((self.points[indices.1] - self.points[indices.0]).length())
    }

    pub fn get_curve_length(&self) -> f32 {
        let mut index = 0;
        let mut total_length = 0.0;
        loop {
            if let Some(length) = self.segment_length(index) {
                total_length += length;
            } else {
                return total_length;
            }
            index += 1;
        }
    }

    pub fn get_segment_direciton(&self, index: usize) -> Option<Vec2> {
        let indices = self.get_segment_point_indices(index);
        let indices = if indices.is_none() {
            return None;
        } else {
            indices.unwrap()
        };

        let start = self.points[indices.0];
        let end = self.points[indices.1];

        let edge_direction = (end - start).normalize();
        let edge_direction = vec2(edge_direction.y, -edge_direction.x);

        Some(edge_direction)
    }

    pub fn get_corner_direction(&self, index: usize) -> Option<Vec2> {
        let directions = if index == 0 {
            if self.close {
                (
                    self.get_segment_direciton(self.last_segment_index())
                        .unwrap(),
                    self.get_segment_direciton(index).unwrap(),
                )
            } else {
                let dir = self.get_corner_direction(index).unwrap();
                (dir, dir)
            }
        } else if index >= self.points.len() {
            return None;
        } else if index == self.points.len() - 1 {
            if self.close {
                (
                    self.get_segment_direciton(index).unwrap(),
                    self.get_segment_direciton(0).unwrap(),
                )
            } else {
                let dir = self.get_segment_direciton(index - 1).unwrap();
                (dir, dir)
            }
        } else {
            (
                self.get_segment_direciton(index - 1).unwrap(),
                self.get_segment_direciton(index).unwrap(),
            )
        };

        let corner_dir = (directions.0 + directions.1) * 0.5;
        let direction_mult = 1.0 / (Vec2::dot(directions.0, corner_dir));
        Some((direction_mult * corner_dir).clamp_length_max(3.0))
    }
}