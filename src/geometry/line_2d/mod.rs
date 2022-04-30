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
    len: f32,
    default_width: f32,
}

impl Line {
    pub fn new(width: f32) -> Self {
        Line {
            list: NeighbourList::new(),
            len: 0.0,
            default_width: width,
        }
    }

    fn from_vecs<T: IntoIterator<Item = Vec2>>(line_width: f32, iter: T) -> Self {
        let mut line = Line::new(line_width);
        for vert in iter {
            line.add(vert);
        }
        line
    }

    pub fn line_length(&self) -> f32 {
        self.len
    }

    pub fn vert_count(&self) -> usize {
        self.list.len()
    }

    pub fn add(&mut self, pos: Vec2) {
        self.add_vert(line_vert_w(pos, self.default_width));
    }

    pub fn add_width(&mut self, pos: Vec2, width: f32) {
        self.add_vert(line_vert_w(pos, width));
    }

    pub fn add_vert(&mut self, vert: LineVertex) {
        self.list.append(vert);

        let prev = self.list.prev(self.list.last().unwrap().idx);
        if let Some(node) = prev {
            self.len += node.val.len;
        }
    }

    pub fn iter(&self) -> NeighbourListValsIter<'_, LineVertex> {
        self.list.vals()
    }
}

impl<'a> IntoIterator for &'a Line {
    type Item = &'a LineVertex;
    type IntoIter = NeighbourListValsIter<'a, LineVertex>;

    fn into_iter(self) -> Self::IntoIter {
        self.list.into_iter()
    }
}

impl FromIterator<LineVertex> for Line {
    fn from_iter<T: IntoIterator<Item = LineVertex>>(iter: T) -> Self {
        let mut line = Line::new(1.0);
        for vert in iter {
            line.add_vert(vert);
        }
        line
    }
}

#[cfg(test)]
mod tests;
