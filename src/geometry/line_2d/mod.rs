use crate::{
    data_structures::neighbour_list::{
        AdjustToNextNeighbour, NeighbourList, NeighbourListValsIter,
    },
    prelude::*,
};
use glam::Vec2;

pub struct LineVertex {
    pub pos: Vec2,
    pub width: f32,
    pub len: f32,
    pub dir: Vec2,
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
    fn adjust_to_next(&mut self, next: &Self) {
        self.point_to(&next.pos);
    }
}

pub fn line_vert(pos: Vec2) -> LineVertex {
    LineVertex::new(pos)
}

pub fn line_vert_w(pos: Vec2, width: f32) -> LineVertex {
    LineVertex {
        pos,
        width,
        ..default()
    }
}

pub struct Line {
    list: NeighbourList<LineVertex>,
    len: usize,
}

impl<'a> IntoIterator for &'a Line {
    type Item = &'a LineVertex;
    type IntoIter = NeighbourListValsIter<'a, LineVertex>;

    fn into_iter(self) -> Self::IntoIter {
        self.list.into_iter()
    }
}

impl Line {
    pub fn new() -> Self {
        Line {
            list: NeighbourList::new(),
            len: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn add(pos: Vec2) {}
}

#[cfg(test)]
mod tests;
