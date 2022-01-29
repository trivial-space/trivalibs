pub trait CoordOps: Copy + Clone {
    fn adjust_coords(&self, x: i32, y: i32, width: usize, height: usize) -> (usize, usize);
}

#[derive(Clone, Copy)]
pub struct NoAdjustCoordOps;
impl CoordOps for NoAdjustCoordOps {
    fn adjust_coords(&self, x: i32, y: i32, _width: usize, _height: usize) -> (usize, usize) {
        (x as usize, y as usize)
    }
}
pub static NO_ADJUST_COORD_OPS: NoAdjustCoordOps = NoAdjustCoordOps {};

#[derive(Clone, Copy)]
pub struct ClampToEdgeCoordOps;
impl CoordOps for ClampToEdgeCoordOps {
    fn adjust_coords(&self, x: i32, y: i32, width: usize, height: usize) -> (usize, usize) {
        let w = width as i32;
        let h = height as i32;
        (x.min(w - 1).max(0) as usize, y.min(h - 1).max(0) as usize)
    }
}
pub static CLAMP_TO_EDGE_COORD_OPS: ClampToEdgeCoordOps = ClampToEdgeCoordOps {};

#[derive(Clone, Copy)]
pub struct CircleAllCoordOps;
impl CoordOps for CircleAllCoordOps {
    fn adjust_coords(&self, x: i32, y: i32, width: usize, height: usize) -> (usize, usize) {
        let w = width as i32;
        let h = height as i32;
        if x < 0 {
            return self.adjust_coords(x + w, y, width, height);
        }
        if y < 0 {
            return self.adjust_coords(x, y + h, width, height);
        }
        if x >= w {
            return self.adjust_coords(x - w, y, width, height);
        }
        if y >= h {
            return self.adjust_coords(x, y - h, width, height);
        }

        (x as usize, y as usize)
    }
}
pub static CIRCLE_ALL_COORD_OPS: CircleAllCoordOps = CircleAllCoordOps {};

pub struct Grid<T, A>
where
    T: Clone + Copy,
    A: CoordOps + Copy + Clone,
{
    pub width: usize,
    pub height: usize,
    vertices: Vec<Vec<T>>,
    coord_ops: A,
}

pub struct Vertex<'a, T, A>
where
    T: Clone + Copy,
    A: CoordOps,
{
    pub x: usize,
    pub y: usize,
    pub val: T,
    grid: &'a Grid<T, A>,
}

pub fn make_grid<T: Clone + Copy>() -> Grid<T, NoAdjustCoordOps> {
    Grid::new(NO_ADJUST_COORD_OPS)
}

pub fn make_grid_with_coord_ops<T: Copy + Clone, A: CoordOps>(coord_ops: A) -> Grid<T, A> {
    Grid::new(coord_ops)
}

impl<T, A> Grid<T, A>
where
    T: Clone + Copy,
    A: CoordOps,
{
    fn new(coord_ops: A) -> Self {
        Grid {
            width: 0,
            height: 0,
            vertices: vec![],
            coord_ops,
        }
    }

    pub fn get(&self, x: i32, y: i32) -> Option<&T> {
        let (x, y) = self.coord_ops.adjust_coords(x, y, self.width, self.height);
        self.vertices.get(x)?.get(y)
    }

    pub fn set(&mut self, x: i32, y: i32, val: T) {
        let (x, y) = self.coord_ops.adjust_coords(x, y, self.width, self.height);

        self.vertices[x][y] = val;
    }

    pub fn add_col(&mut self, vals: Vec<T>) {
        let len = vals.len();
        if self.height == 0 {
            self.height = len;
        } else if self.height > len {
            panic!("new column length needs to be at least as big as the grid height.")
        }

        self.vertices.push(vals);
        self.width += 1;
    }

    pub fn add_row(&mut self, vals: Vec<T>) {
        let len = vals.len();
        if self.width == 0 {
            self.width = len;
            for _i in 0..len {
                self.vertices.push(vec![]);
            }
        } else if self.width > len {
            panic!("new row length needs to be at least as big as the grid width.");
        }

        self.height += 1;
        for i in 0..self.width {
            self.vertices[i].push(vals[i]);
        }
    }

    pub fn vertex(&self, x: i32, y: i32) -> Option<Vertex<T, A>> {
        let (x, y) = self.coord_ops.adjust_coords(x, y, self.width, self.height);
        self.vertices.get(x)?.get(y).map(|val| Vertex {
            x,
            y,
            grid: self,
            val: *val,
        })
    }

    pub fn map<B, F>(&self, f: F) -> Grid<B, A>
    where
        B: Clone + Copy,
        F: Fn(Vertex<T, A>) -> B,
    {
        let mut grid = Grid::new(self.coord_ops);
        for x in 0..self.width as i32 {
            let mut col = vec![];
            for y in 0..self.height as i32 {
                col.push(f(self.vertex(x, y).unwrap()));
            }
            grid.add_col(col);
        }
        grid
    }
}

impl<T: Copy, A: CoordOps> PartialEq for Vertex<'_, T, A> {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl<T, A> Vertex<'_, T, A>
where
    T: Clone + Copy,
    A: CoordOps,
{
    pub fn next(&self, x_offset: i32, y_offset: i32) -> Option<Self> {
        self.grid
            .vertex(self.x as i32 + x_offset, self.y as i32 + y_offset)
            .and_then(|vert| if vert != *self { Some(vert) } else { None })
    }
    pub fn left(&self) -> Option<Self> {
        self.next(-1, 0)
    }
    pub fn right(&self) -> Option<Self> {
        self.next(1, 0)
    }
    pub fn top(&self) -> Option<Self> {
        self.next(0, -1)
    }
    pub fn bottom(&self) -> Option<Self> {
        self.next(0, 1)
    }
}

#[cfg(test)]
mod tests;
