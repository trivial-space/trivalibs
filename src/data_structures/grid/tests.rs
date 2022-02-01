use super::*;

#[derive(Clone, Copy, PartialEq, Debug)]
struct Coord(i32, i32);

fn fill_grid<C: CoordOps>(mut grid: Grid<Coord, C>) -> Grid<Coord, C> {
    for x in 0..3 {
        let mut col = vec![];
        for y in 0..3 {
            col.push(Coord(x, y));
        }
        grid.add_col(col);
    }
    grid
}

#[test]
fn get_set_no_adjust() {
    let mut grid = fill_grid(make_grid());
    assert_eq!(grid.get(1, 1), Some(&Coord(1, 1)));
    assert_eq!(grid.get(4, 4), None);
    assert_eq!(grid.get(1, -1), None);
    assert_eq!(grid.get(-1, 1), None);

    grid.set(1, 1, Coord(5, 5));

    assert_eq!(grid.get(1, 1), Some(&Coord(5, 5)));
}

#[test]
fn get_set_clamp() {
    let mut grid = fill_grid(make_grid_with_coord_ops(CLAMP_TO_EDGE_COORD_OPS));

    assert_eq!(grid.get(1, 1).unwrap(), &Coord(1, 1));
    assert_eq!(grid.get(4, 4).unwrap(), &Coord(2, 2));
    assert_eq!(grid.get(-2, -2).unwrap(), &Coord(0, 0));

    grid.set(1, 1, Coord(5, 5));
    assert_eq!(grid.get(1, 1).unwrap(), &Coord(5, 5));
    assert_eq!(grid.get(4, 4).unwrap(), &Coord(2, 2));
    assert_eq!(grid.get(-2, -2).unwrap(), &Coord(0, 0));

    grid.set(-1, -1, Coord(6, 6));
    assert_eq!(grid.get(0, 0).unwrap(), &Coord(6, 6));

    grid.set(4, 4, Coord(7, 7));
    assert_eq!(grid.get(2, 2).unwrap(), &Coord(7, 7));
}

#[test]
fn get_set_circle_all() {
    let mut grid = fill_grid(make_grid_with_coord_ops(CIRCLE_ALL_COORD_OPS));

    assert_eq!(grid.get(1, 1).unwrap(), &Coord(1, 1));
    assert_eq!(grid.get(4, 4).unwrap(), &Coord(1, 1));
    assert_eq!(grid.get(-2, -2).unwrap(), &Coord(1, 1));
    assert_eq!(grid.get(-1, -1).unwrap(), &Coord(2, 2));

    grid.set(1, 1, Coord(5, 5));
    assert_eq!(grid.get(1, 1).unwrap(), &Coord(5, 5));
    assert_eq!(grid.get(4, 4).unwrap(), &Coord(5, 5));
    assert_eq!(grid.get(-2, -2).unwrap(), &Coord(5, 5));

    grid.set(-1, -1, Coord(6, 6));
    assert_eq!(grid.get(2, 2).unwrap(), &Coord(6, 6));

    grid.set(4, 4, Coord(7, 7));
    assert_eq!(grid.get(1, 1).unwrap(), &Coord(7, 7));
}

#[test]
fn fill_grid_rows_cols() {
    let mut grid1 = make_grid();
    assert_eq!(grid1.width, 0);
    assert_eq!(grid1.height, 0);

    grid1.add_col(vec![Coord(0, 0), Coord(0, 1), Coord(0, 2)]);
    assert_eq!(grid1.width, 1);
    assert_eq!(grid1.height, 3);
    assert_eq!(*grid1.get(0, 2).unwrap(), Coord(0, 2));

    grid1.add_row(vec![Coord(0, 3), Coord(1, 3)]);
    assert_eq!(grid1.width, 1);
    assert_eq!(grid1.height, 4);
    assert_eq!(*grid1.get(0, 3).unwrap(), Coord(0, 3));
    assert_eq!(grid1.get(1, 3), None);

    let mut grid2 = make_grid();
    assert_eq!(grid2.width, 0);
    assert_eq!(grid2.height, 0);

    grid2.add_row(vec![Coord(0, 0), Coord(1, 0), Coord(2, 0)]);
    assert_eq!(grid2.width, 3);
    assert_eq!(grid2.height, 1);
    assert_eq!(*grid2.get(1, 0).unwrap(), Coord(1, 0));
    assert_eq!(*grid2.get(2, 0).unwrap(), Coord(2, 0));

    grid2.add_col(vec![Coord(3, 0), Coord(3, 1)]);
    assert_eq!(grid2.width, 4);
    assert_eq!(grid2.height, 1);
    assert_eq!(*grid2.get(3, 0).unwrap(), Coord(3, 0));
    assert_eq!(grid2.get(3, 1), None);
}

#[test]
fn grid_vertices() {
    let grid = fill_grid(make_grid_with_coord_ops(CIRCLE_ALL_COORD_OPS));
    let v = grid.vertex(0, 0).unwrap();

    assert_eq!(v.x, 0);
    assert_eq!(v.y, 0);
    assert_eq!(v.val, Coord(0, 0));

    let v = v.right().unwrap();
    assert_eq!(v.x, 1);
    assert_eq!(v.y, 0);
    assert_eq!(v.val, Coord(1, 0));

    let v = v.right().unwrap();
    assert_eq!(v.x, 2);
    assert_eq!(v.y, 0);
    assert_eq!(v.val, Coord(2, 0));

    let v = v.right().unwrap();
    assert_eq!(v.x, 0);
    assert_eq!(v.y, 0);
    assert_eq!(v.val, Coord(0, 0));

    let v = v.bottom().unwrap();
    assert_eq!(v.x, 0);
    assert_eq!(v.y, 1);
    assert_eq!(v.val, Coord(0, 1));

    let v = v.bottom().unwrap();
    assert_eq!(v.x, 0);
    assert_eq!(v.y, 2);
    assert_eq!(v.val, Coord(0, 2));

    let v = v.bottom().unwrap();
    assert_eq!(v.x, 0);
    assert_eq!(v.y, 0);
    assert_eq!(v.val, Coord(0, 0));

    let v = v.top().unwrap();
    assert_eq!(v.x, 0);
    assert_eq!(v.y, 2);
    assert_eq!(v.val, Coord(0, 2));

    let v = v.left().unwrap();
    assert_eq!(v.x, 2);
    assert_eq!(v.y, 2);
    assert_eq!(v.val, Coord(2, 2));

    let v = v.top().unwrap();
    assert_eq!(v.x, 2);
    assert_eq!(v.y, 1);
    assert_eq!(v.val, Coord(2, 1));

    let v = v.left().unwrap();
    assert_eq!(v.x, 1);
    assert_eq!(v.y, 1);
    assert_eq!(v.val, Coord(1, 1));
}

#[test]
fn test_grid_map() {
    let grid1 = fill_grid(make_grid());
    let grid2 = grid1.map(|vert| Coord(vert.val.0 + vert.x as i32, vert.val.1 + vert.y as i32));
    assert_eq!(*grid2.get(0, 0).unwrap(), Coord(0, 0));
    assert_eq!(*grid2.get(1, 1).unwrap(), Coord(2, 2));
    assert_eq!(*grid1.get(1, 1).unwrap(), Coord(1, 1));
    assert_eq!(*grid2.get(2, 2).unwrap(), Coord(4, 4));
    assert_eq!(*grid1.get(2, 2).unwrap(), Coord(2, 2));
    assert_eq!(*grid2.get(2, 1).unwrap(), Coord(4, 2));
}

#[test]
fn rows_and_cols() {
    let grid = fill_grid(make_grid());
    let col = grid.col(1).unwrap();
    assert_eq!(*col, vec![Coord(1, 0), Coord(1, 1), Coord(1, 2)]);

    let row = grid.row(1).unwrap();
    assert_eq!(row, vec![Coord(0, 1), Coord(1, 1), Coord(2, 1)]);
}

#[test]
fn flat_map() {
    #[derive(Clone, Copy, PartialEq, Debug)]
    struct CoordF(f32, f32);

    let grid = fill_grid(make_grid_with_coord_ops(CIRCLE_ALL_COORD_OPS));
    let grid2 = grid.flat_map_cols(|col| {
        let col2 = col
            .iter()
            .map(|vert| {
                CoordF(
                    (vert.val.0 as f32 + vert.right().unwrap().val.0 as f32) / 2.0,
                    vert.val.1 as f32,
                )
            })
            .collect();

        vec![
            col.into_iter()
                .map(|vert| CoordF(vert.val.0 as f32, vert.val.1 as f32))
                .collect(),
            col2,
        ]
    });

    assert_eq!(grid2.width, 6);
    assert_eq!(grid2.height, 3);
    for y in 0..grid2.height as i32 {
        assert_eq!(*grid2.get(0, y).unwrap(), CoordF(0.0, y as f32));
        assert_eq!(*grid2.get(1, y).unwrap(), CoordF(0.5, y as f32));
        assert_eq!(*grid2.get(2, y).unwrap(), CoordF(1.0, y as f32));
        assert_eq!(*grid2.get(3, y).unwrap(), CoordF(1.5, y as f32));
        assert_eq!(*grid2.get(4, y).unwrap(), CoordF(2.0, y as f32));
        assert_eq!(*grid2.get(5, y).unwrap(), CoordF(1.0, y as f32));
    }

    let grid2 = grid.flat_map_rows(|row| {
        let row2 = row
            .iter()
            .map(|vert| {
                CoordF(
                    vert.val.0 as f32,
                    (vert.val.1 as f32 + vert.bottom().unwrap().val.1 as f32) / 2.0,
                )
            })
            .collect();

        vec![
            row.into_iter()
                .map(|vert| CoordF(vert.val.0 as f32, vert.val.1 as f32))
                .collect(),
            row2,
        ]
    });

    assert_eq!(grid2.width, 3);
    assert_eq!(grid2.height, 6);
    for x in 0..grid2.width as i32 {
        assert_eq!(*grid2.get(x, 0).unwrap(), CoordF(x as f32, 0.0));
        assert_eq!(*grid2.get(x, 1).unwrap(), CoordF(x as f32, 0.5));
        assert_eq!(*grid2.get(x, 2).unwrap(), CoordF(x as f32, 1.0));
        assert_eq!(*grid2.get(x, 3).unwrap(), CoordF(x as f32, 1.5));
        assert_eq!(*grid2.get(x, 4).unwrap(), CoordF(x as f32, 2.0));
        assert_eq!(*grid2.get(x, 5).unwrap(), CoordF(x as f32, 1.0));
    }
}
