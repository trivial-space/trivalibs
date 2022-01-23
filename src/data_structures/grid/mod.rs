pub trait CoordOps {
    fn adjust_coords(&self, x: i32, y: i32, width: usize, height: usize) -> (usize, usize);
}

#[derive(Clone, Copy)]
pub struct DefaultCoordOps;
impl CoordOps for DefaultCoordOps {
    fn adjust_coords(&self, x: i32, y: i32, width: usize, height: usize) -> (usize, usize) {
        let w = width as i32;
        let h = height as i32;
        (x.min(w - 1) as usize, y.min(h - 1) as usize)
    }
}

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
    A: CoordOps + Copy + Clone,
{
    pub x: usize,
    pub y: usize,
    pub val: T,
    grid: &'a Grid<T, A>,
}

pub fn make_grid<B: Clone + Copy>() -> Grid<B, DefaultCoordOps> {
    let cells = vec![];
    Grid {
        width: 0,
        height: 0,
        vertices: cells,
        coord_ops: DefaultCoordOps {},
    }
}

impl<T, A> Grid<T, A>
where
    T: Clone + Copy,
    A: CoordOps + Copy + Clone,
{
    pub fn new(coord_ops: A) -> Self {
        let cells = vec![];
        Grid {
            width: 0,
            height: 0,
            vertices: cells,
            coord_ops,
        }
    }

    pub fn get(&self, x: i32, y: i32) -> T {
        let (x, y) = self.coord_ops.adjust_coords(x, y, self.width, self.height);

        self.vertices[x][y]
    }

    pub fn set(&mut self, x: i32, y: i32, val: T) {
        let (x, y) = self.coord_ops.adjust_coords(x, y, self.width, self.height);

        self.vertices[x][y] = val;
    }

    pub fn add_col(&mut self, vals: Vec<T>) {
        if self.height == 0 {
            self.height = vals.len();
        } else if self.height > vals.len() {
            panic!("new column length needs to be at least as big as the grid height.")
        }

        self.vertices.push(vals);
        self.width += 1;
    }

    pub fn add_row(&mut self, vals: Vec<T>) {
        if self.width == 0 {
            panic!("grid needs at least one column to add more rows.");
        } else if self.width > vals.len() {
            panic!("new row length needs to be at least as big as the grid width.");
        }

        for i in 0..self.width {
            self.vertices[i].push(vals[i]);
        }
    }

    pub fn vertex(&self, x: i32, y: i32) -> Vertex<T, A> {
        let (x, y) = self.coord_ops.adjust_coords(x, y, self.width, self.height);

        Vertex {
            x,
            y,
            grid: self,
            val: self.vertices[x][y],
        }
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
                col.push(f(self.vertex(x, y)));
            }
            grid.add_col(col);
        }
        grid
    }
}

impl<T: Copy, A: CoordOps + Copy> PartialEq for Vertex<'_, T, A> {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }

    fn ne(&self, other: &Self) -> bool {
        self.x != other.x || self.y != other.y
    }
}

impl<T, A> Vertex<'_, T, A>
where
    T: Clone + Copy,
    A: CoordOps + Copy + Clone,
{
    pub fn left(&self) -> Option<Self> {
        let vert = self.grid.vertex(self.x as i32 - 1, self.y as i32);
        if vert != *self {
            Some(vert)
        } else {
            None
        }
    }
    pub fn right(&self) -> Option<Self> {
        let vert = self.grid.vertex(self.x as i32 + 1, self.y as i32);
        if vert != *self {
            Some(vert)
        } else {
            None
        }
    }
    pub fn top(&self) -> Option<Self> {
        let vert = self.grid.vertex(self.x as i32, self.y as i32 - 1);
        if vert != *self {
            Some(vert)
        } else {
            None
        }
    }
    pub fn bottom(&self) -> Option<Self> {
        let vert = self.grid.vertex(self.x as i32, self.y as i32 + 1);
        if vert != *self {
            Some(vert)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests;
