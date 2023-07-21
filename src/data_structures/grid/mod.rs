pub trait CoordOpsFn: Copy + Clone {
    fn adjust_coords(&self, x: i32, y: i32, width: usize, height: usize) -> (usize, usize);
    fn circle(&self) -> (bool, bool);
}

#[derive(Clone, Copy)]
pub struct ClampToEdgeCoordOps;
impl CoordOpsFn for ClampToEdgeCoordOps {
    fn adjust_coords(&self, x: i32, y: i32, width: usize, height: usize) -> (usize, usize) {
        let w = width as i32;
        let h = height as i32;
        (x.min(w - 1).max(0) as usize, y.min(h - 1).max(0) as usize)
    }
    fn circle(&self) -> (bool, bool) {
        (false, false)
    }
}
pub static CLAMP_TO_EDGE_COORD_OPS: ClampToEdgeCoordOps = ClampToEdgeCoordOps {};

#[derive(Clone, Copy)]
pub struct CircleRowsCoordOps;
impl CoordOpsFn for CircleRowsCoordOps {
    fn adjust_coords(&self, x: i32, y: i32, width: usize, height: usize) -> (usize, usize) {
        let h = height as i32;
        if y < 0 {
            return self.adjust_coords(x, y + h, width, height);
        }
        if y >= h {
            return self.adjust_coords(x, y - h, width, height);
        }

        (x as usize, y as usize)
    }
    fn circle(&self) -> (bool, bool) {
        (false, true)
    }
}
pub static CIRCLE_ROWS_COORD_OPS: CircleRowsCoordOps = CircleRowsCoordOps {};

#[derive(Clone, Copy)]
pub struct CircleColsCoordOps;
impl CoordOpsFn for CircleColsCoordOps {
    fn adjust_coords(&self, x: i32, y: i32, width: usize, height: usize) -> (usize, usize) {
        let w = width as i32;
        if x < 0 {
            return self.adjust_coords(x + w, y, width, height);
        }
        if x >= w {
            return self.adjust_coords(x - w, y, width, height);
        }

        (x as usize, y as usize)
    }
    fn circle(&self) -> (bool, bool) {
        (true, false)
    }
}
pub static CIRCLE_COLS_COORD_OPS: CircleColsCoordOps = CircleColsCoordOps {};

#[derive(Clone, Copy)]
pub struct CircleAllCoordOps;
impl CoordOpsFn for CircleAllCoordOps {
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
    fn circle(&self) -> (bool, bool) {
        (true, false)
    }
}
pub static CIRCLE_ALL_COORD_OPS: CircleAllCoordOps = CircleAllCoordOps {};

pub struct Grid<T, A>
where
    T: Clone + Copy,
    A: CoordOpsFn,
{
    pub width: usize,
    pub height: usize,
    vertices: Vec<Vec<T>>,
    coord_ops: A,
}

pub struct Vertex<'a, T, A>
where
    T: Clone + Copy,
    A: CoordOpsFn,
{
    pub x: usize,
    pub y: usize,
    pub val: T,
    grid: &'a Grid<T, A>,
}

pub fn make_grid<T: Clone + Copy>() -> Grid<T, ClampToEdgeCoordOps> {
    Grid::new(CLAMP_TO_EDGE_COORD_OPS)
}

pub fn make_grid_with_coord_ops<T: Copy + Clone, A: CoordOpsFn>(coord_ops: A) -> Grid<T, A> {
    Grid::new(coord_ops)
}

impl<T, A> Grid<T, A>
where
    T: Clone + Copy,
    A: CoordOpsFn,
{
    fn new(coord_ops: A) -> Self {
        Grid {
            width: 0,
            height: 0,
            vertices: vec![],
            coord_ops,
        }
    }

    pub fn get(&self, x: i32, y: i32) -> &T {
        let (x, y) = self.coord_ops.adjust_coords(x, y, self.width, self.height);
        &self.vertices[x][y]
    }

    pub fn set(&mut self, x: i32, y: i32, val: T) {
        let (x, y) = self.coord_ops.adjust_coords(x, y, self.width, self.height);

        self.vertices[x][y] = val;
    }

    pub fn vertex(&self, x: i32, y: i32) -> Vertex<T, A> {
        let (x, y) = self.coord_ops.adjust_coords(x, y, self.width, self.height);
        let val = self.vertices[x][y];
        Vertex {
            x,
            y,
            grid: self,
            val: val,
        }
    }

    pub fn col(&self, x: i32) -> &Vec<T> {
        let (new_x, _) = self.coord_ops.adjust_coords(x, 0, self.width, self.height);
        &self.vertices[new_x]
    }

    pub fn row(&self, y: i32) -> Vec<T> {
        let (_, new_y) = self.coord_ops.adjust_coords(0, y, self.width, self.height);
        let mut row = vec![];
        for x in 0..self.width {
            row.push(self.vertices[x][new_y]);
        }
        row
    }

    pub fn add_col(&mut self, vals: Vec<T>) {
        let len = vals.len();
        if self.height == 0 {
            self.height = len;
        } else if self.height != len {
            panic!("new column length needs to be as big as the grid height.")
        }

        self.width += 1;
        self.vertices.push(vals);
    }

    pub fn add_row(&mut self, vals: Vec<T>) {
        let len = vals.len();
        if self.width == 0 {
            self.width = len;
            for _i in 0..len {
                self.vertices.push(vec![]);
            }
        } else if self.width != len {
            panic!("new row length needs to be as big as the grid width.");
        }

        self.height += 1;
        for i in 0..self.width {
            self.vertices[i].push(vals[i]);
        }
    }

    pub fn map<B, F>(&self, f: F) -> Grid<B, A>
    where
        B: Clone + Copy,
        F: Fn(Vertex<T, A>) -> B,
    {
        let mut grid = Grid::new(self.coord_ops);
        for x in 0..self.width {
            let mut col = vec![];
            for y in 0..self.height {
                col.push(f(self.vertex(x as i32, y as i32)));
            }
            grid.add_col(col);
        }
        grid
    }

    pub fn connected_border_count(&self) -> (usize, usize) {
        let (circle_cols, circle_rows) = self.coord_ops.circle();
        let w = if circle_cols {
            self.width
        } else {
            self.width - 1
        };
        let h = if circle_rows {
            self.height
        } else {
            self.height - 1
        };
        (w, h)
    }

    pub fn flat_map_cols<B, F>(&self, f: F) -> Grid<B, A>
    where
        B: Clone + Copy,
        F: Fn(Vec<Vertex<T, A>>) -> Vec<Vec<B>>,
    {
        let mut grid = make_grid_with_coord_ops(self.coord_ops);
        for x in 0..self.width {
            let mut col = vec![];
            for y in 0..self.height {
                col.push(self.vertex(x as i32, y as i32));
            }
            let new_colls = f(col);
            for i in 0..new_colls.len() {
                grid.add_col(new_colls[i].to_vec());
            }
        }
        grid
    }

    pub fn flat_map_rows<B, F>(&self, f: F) -> Grid<B, A>
    where
        B: Clone + Copy,
        F: Fn(Vec<Vertex<T, A>>) -> Vec<Vec<B>>,
    {
        let mut grid = make_grid_with_coord_ops(self.coord_ops);
        for y in 0..self.height {
            let mut row = vec![];
            for x in 0..self.width {
                row.push(self.vertex(x as i32, y as i32));
            }
            let new_rows = f(row);
            for i in 0..new_rows.len() {
                grid.add_row(new_rows[i].to_vec());
            }
        }
        grid
    }

    pub fn to_cw_quads<'a>(&self) -> Vec<[T; 4]> {
        let (w, h) = self.connected_border_count();
        let mut quads = vec![];
        for x in 0..w {
            for y in 0..h {
                let v = self.vertex(x as i32, y as i32);
                quads.push([
                    v.val,
                    v.right().unwrap().val,
                    v.right().unwrap().bottom().unwrap().val,
                    v.bottom().unwrap().val,
                ])
            }
        }
        quads
    }

    pub fn to_ccw_quads<'a>(&self) -> Vec<[T; 4]> {
        let (w, h) = self.connected_border_count();
        let mut quads = vec![];
        for x in 0..w {
            for y in 0..h {
                let v = self.vertex(x as i32, y as i32);
                quads.push([
                    v.val,
                    v.bottom().unwrap().val,
                    v.right().unwrap().bottom().unwrap().val,
                    v.right().unwrap().val,
                ])
            }
        }
        quads
    }

    pub fn subdivide<F: Fn(T, T, f32) -> T>(&self, count_x: u32, count_y: u32, lerp: F) {
        let (circle_x, circle_y) = self.coord_ops.circle();
        let grid1 = self.flat_map_cols(|col| {
            let col1 = col.into_iter().map(|v| v.val).collect();
            for i in 0..count_x {}
        });
    }
}

impl<T: Copy, A: CoordOpsFn> PartialEq for Vertex<'_, T, A> {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl<T, A> Vertex<'_, T, A>
where
    T: Clone + Copy,
    A: CoordOpsFn,
{
    pub fn next(&self, x_offset: i32, y_offset: i32) -> Option<Self> {
        let vert = self
            .grid
            .vertex(self.x as i32 + x_offset, self.y as i32 + y_offset);
        if vert != *self {
            Some(vert)
        } else {
            None
        }
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
