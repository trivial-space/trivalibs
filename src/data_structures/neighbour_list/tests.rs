use super::{AdjustToNextNeighbour, NeighbourList};

#[derive(Debug, PartialEq, Clone, Copy)]
struct Item {
    idx: u8,
    next: u8,
}

impl AdjustToNextNeighbour for Item {
    fn adjust_to_next(&mut self, next: &Self) {
        self.next = next.idx
    }
}

fn item(idx: u8) -> Item {
    Item { idx, next: 0 }
}
fn with_next(idx: u8) -> Item {
    Item { idx, next: idx + 1 }
}

#[test]
fn create_append_and_iter() {
    let mut list = NeighbourList::new();

    assert_eq!(list.first(), None);
    assert_eq!(list.last(), None);
    assert_eq!(list.iter().nth(0), None);
    assert_eq!(list.iter().nth_back(0), None);

    let item1 = item(1);
    list.append(item1);

    assert_eq!(list.first().unwrap().val, item1);
    assert_eq!(list.last().unwrap().val, item1);
    assert_eq!(list.iter().nth(0).unwrap().val, item1);
    assert_eq!(list.iter().nth_back(0).unwrap().val, item1);

    let item2 = item(2);
    list.append(item2);

    assert_eq!(list.first().unwrap().val, with_next(1));
    assert_eq!(list.last().unwrap().val, item2);

    let item3 = item(3);
    list.append(item3);

    assert_eq!(list.first().unwrap().val, with_next(1));
    assert_eq!(list.last().unwrap().val, item3);

    assert_eq!(list.next(list.first().unwrap()).unwrap().val, with_next(2));
    assert_eq!(
        list.next(list.next(list.first().unwrap()).unwrap())
            .unwrap()
            .val,
        item3
    );

    assert_eq!(list.prev(list.last().unwrap()).unwrap().val, with_next(2));
    assert_eq!(
        list.prev(list.prev(list.last().unwrap()).unwrap())
            .unwrap()
            .val,
        with_next(1)
    );

    assert_eq!(list.iter().nth(0).unwrap().val, with_next(1));
    assert_eq!(list.iter().nth(1).unwrap().val, with_next(2));
    assert_eq!(list.iter().nth(2).unwrap().val, item3);

    assert_eq!(list.iter().nth_back(0).unwrap().val, item3);
    assert_eq!(list.iter().nth_back(1).unwrap().val, with_next(2));
    assert_eq!(list.iter().nth_back(2).unwrap().val, with_next(1));
}

#[test]
fn mutable_iterator() {
    let mut list = NeighbourList::new();

    let item1 = item(1);
    let item2 = item(2);
    let item3 = item(3);

    list.append(item1);
    list.append(item2);
    list.append(item3);

    for item in list.iter_mut() {
        item.val.idx *= 2;
    }

    assert_eq!(list.iter().nth(0).unwrap().val.idx, 2);
    assert_eq!(list.iter().nth(1).unwrap().val.idx, 4);
    assert_eq!(list.iter().nth(2).unwrap().val.idx, 6);
}
