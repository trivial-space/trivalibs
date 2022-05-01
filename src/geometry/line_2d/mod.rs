use crate::prelude::*;
use glam::Vec2;
use std::slice::Iter;

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
    list: Vec<LineVertex>,
    len: f32,
    default_width: f32,
}

impl Line {
    pub fn new(width: f32) -> Self {
        Line {
            list: Vec::new(),
            len: 0.0,
            default_width: width,
        }
    }

    pub fn from_vecs<T: IntoIterator<Item = Vec2>>(line_width: f32, iter: T) -> Self {
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

    pub fn add_vert(&mut self, mut vert: LineVertex) {
        let curr_len = self.list.len();

        if curr_len > 0 {
            let idx = curr_len - 1;
            let prev = &mut self.list[idx];
            prev.point_to(&vert.pos);

            self.len += prev.len;
            vert.dir = prev.dir;
        }

        self.list.push(vert);
    }

    pub fn iter(&self) -> Iter<'_, LineVertex> {
        self.list.iter()
    }

    pub fn get(&self, i: usize) -> &LineVertex {
        &self.list[i]
    }

    pub fn last(&self) -> &LineVertex {
        &self.list[self.list.len() - 1]
    }
}

impl<'a> IntoIterator for &'a Line {
    type Item = &'a LineVertex;
    type IntoIter = Iter<'a, LineVertex>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
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
