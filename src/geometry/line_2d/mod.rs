use crate::{data_structures::neighbour_list::AdjustToNextNeighbour, prelude::*};
use glam::Vec2;

pub struct LineVertex {
    pos: Vec2,
    width: f32,
    len: f32,
    dir: Vec2,
}

impl Default for LineVertex {
    fn default() -> Self {
        Self {
            pos: Vec2::ZERO,
            width: 1.0,
            len: 0.0,
            dir: Vec2::ZERO,
        }
    }
}

impl LineVertex {
    #[inline]
    fn new(pos: Vec2) -> Self {
        LineVertex { pos, ..default() }
    }

    fn point_to(&mut self, point: &Vec2) {
        let mut vec = *point - self.pos;
        let len = vec.length();
        self.len = len;
        vec /= len;
        self.dir = vec;
    }
}

impl AdjustToNextNeighbour for LineVertex {
    #[inline]
    fn adjust_to_next(&mut self, next: &Self) {
        self.point_to(&next.pos);
    }
}

#[inline]
pub fn line_vert(pos: Vec2) -> LineVertex {
    LineVertex::new(pos)
}

#[inline]
pub fn line_vert_w(pos: Vec2, width: f32) -> LineVertex {
    LineVertex {
        pos,
        width,
        ..default()
    }
}
