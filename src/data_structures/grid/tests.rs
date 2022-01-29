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

// #[test]
// fn frame_cells() {
//     use TestData::*;

//     let mut f = Grid::new(3, 3);
//     let c1 = f.vertex(1, 1);

//     assert_eq!(c1.val, One);
//     assert_eq!(c1.x, 1);
//     assert_eq!(c1.y, 1);
//     assert_eq!(c1.top().val, One);
//     assert_eq!(c1.bottom().val, One);
//     assert_eq!(c1.right().val, One);
//     assert_eq!(c1.left().val, One);
//     assert_eq!(c1.right().x, 2);

//     f.set(1, 1, Three);
//     f.set(0, 1, Two);
//     f.set(1, 0, Three);
//     f.set(2, 1, Two);
//     f.set(1, 2, Three);
//     let c2 = f.vertex(1, 1);

//     assert_eq!(c2.val, Three);
//     assert_eq!(c2.top().val, Three);
//     assert_eq!(c2.bottom().val, Three);
//     assert_eq!(c2.right().val, Two);
//     assert_eq!(c2.left().val, Two);
// }

// #[test]
// fn can_map() {
//     use TestData::*;

//     let mut f = Grid::new(2, 2);
//     f.set(0, 0, Three);
//     f.set(0, 1, Three);

//     let f2 = f.map(|cell| if cell.val == Three { One } else { Three });

//     assert_eq!(f2.get(0, 0), One);
//     assert_eq!(f2.get(0, 1), One);
//     assert_eq!(f2.get(1, 0), Three);
//     assert_eq!(f2.get(1, 1), Three);

//     let f3 = f.map(|cell| cell.right().val);

//     assert_eq!(f3.get(0, 0), One);
//     assert_eq!(f3.get(0, 1), One);
//     assert_eq!(f3.get(1, 0), Three);
//     assert_eq!(f3.get(1, 1), Three);
// }
