pub struct Grid<T: Default + Clone + Copy> {
    pub width: u16,
    pub height: u16,
    cells: Vec<T>,
}

pub struct Vertex<'a, T: Default + Clone + Copy> {
    pub x: u16,
    pub y: u16,
    pub val: T,
    frame: &'a Grid<T>,
}

impl<T: Default + Clone + Copy> Grid<T> {
    pub fn new(width: u16, height: u16) -> Self {
        let cells = vec![T::default(); (width * height).into()];
        let frame = Grid {
            width,
            height,
            cells,
        };

        frame
    }

    pub fn get(&self, x: i32, y: i32) -> T {
        let (x, y) = adjust_coords(x, y, self.width, self.height);

        self.cells[index(y, x, self.height)]
    }

    pub fn set(&mut self, x: i32, y: i32, val: T) {
        let (x, y) = adjust_coords(x, y, self.width, self.height);

        self.cells[index(y, x, self.height)] = val
    }

    pub fn vertex(&self, x: i32, y: i32) -> Vertex<T> {
        let (x, y) = adjust_coords(x, y, self.width, self.height);

        Vertex {
            x,
            y,
            frame: self,
            val: self.cells[index(y, x, self.height)],
        }
    }

    pub fn map<F>(&self, f: F) -> Self
    where
        F: Fn(Vertex<T>) -> T,
    {
        let mut grid = Grid::new(self.width, self.height);
        for x in 0..self.width as i32 {
            for y in 0..self.height as i32 {
                grid.set(x, y, f(self.vertex(x, y)));
            }
        }
        grid
    }
}

impl<T: Default + Clone + Copy> Vertex<'_, T> {
    pub fn left(&self) -> Self {
        self.frame.vertex(self.x as i32 - 1, self.y as i32)
    }
    pub fn right(&self) -> Self {
        self.frame.vertex(self.x as i32 + 1, self.y as i32)
    }
    pub fn top(&self) -> Self {
        self.frame.vertex(self.x as i32, self.y as i32 - 1)
    }
    pub fn bottom(&self) -> Self {
        self.frame.vertex(self.x as i32, self.y as i32 + 1)
    }
}

fn index(y: u16, x: u16, height: u16) -> usize {
    (y * height + x) as usize
}

fn adjust_coords(x: i32, y: i32, width: u16, height: u16) -> (u16, u16) {
    let w = width as i32;
    let h = height as i32;
    if x < 0 {
        return adjust_coords(x + w, y, width, height);
    }
    if y < 0 {
        return adjust_coords(x, y + h, width, height);
    }
    if x >= w {
        return adjust_coords(x - w, y, width, height);
    }
    if y >= h {
        return adjust_coords(x, y - h, width, height);
    }

    (x as u16, y as u16)
}

#[cfg(test)]
mod tests;
