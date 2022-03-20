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

#[test]
fn create_append_and_iter() {
    let mut list = NeighbourList::new();

    assert_eq!(list.first(), None);
    assert_eq!(list.last(), None);
    assert_eq!(list.iter().nth(0), None);
    assert_eq!(list.iter().nth_back(0), None);

    let first_item = item(1);
    list.append(first_item);

    assert_eq!(list.first().unwrap().val, first_item);
    assert_eq!(list.last().unwrap().val, first_item);
    assert_eq!(list.iter().nth(0).unwrap().val, first_item);
    assert_eq!(list.iter().nth_back(0).unwrap().val, first_item);

    let second_item = item(2);
    list.append(second_item);

    assert_eq!(list.first().unwrap().val, first_item);
    assert_eq!(list.last().unwrap().val, second_item);

    let third_item = item(3);
    list.append(third_item);

    assert_eq!(list.first().unwrap().val, first_item);
    assert_eq!(list.last().unwrap().val, third_item);

    assert_eq!(list.next(list.first().unwrap()).unwrap().val, second_item);
    assert_eq!(
        list.next(list.next(list.first().unwrap()).unwrap())
            .unwrap()
            .val,
        third_item
    );

    assert_eq!(list.prev(list.last().unwrap()).unwrap().val, second_item);
    assert_eq!(
        list.prev(list.prev(list.last().unwrap()).unwrap())
            .unwrap()
            .val,
        first_item
    );

    assert_eq!(list.iter().nth(0).unwrap().val, first_item);
    assert_eq!(list.iter().nth(1).unwrap().val, second_item);
    assert_eq!(list.iter().nth(2).unwrap().val, third_item);

    assert_eq!(list.iter().nth_back(0).unwrap().val, third_item);
    assert_eq!(list.iter().nth_back(1).unwrap().val, second_item);
    assert_eq!(list.iter().nth_back(2).unwrap().val, first_item);
}

#[test]
fn mutable_iterator() {
    let mut list = NeighbourList::new();

    let first_item = item(1);
    let second_item = item(2);
    let third_item = item(3);
    list.append(first_item);
    list.append(second_item);
    list.append(third_item);

    for item in list.iter_mut() {
        item.val.idx *= 2;
    }
    assert_eq!(list.iter().nth_back(0).unwrap().val.idx, 6);
    assert_eq!(list.iter().nth_back(1).unwrap().val.idx, 4);
    assert_eq!(list.iter().nth_back(2).unwrap().val.idx, 2);
}
