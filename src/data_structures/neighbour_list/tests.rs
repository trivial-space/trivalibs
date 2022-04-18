use super::traits::NeighbourMapTransform;
use super::{AdjustToNextNeighbour, NeighbourList};

#[derive(Debug, PartialEq, Clone, Copy)]
struct Item {
    number: u8,
    adjusted: u8,
}

impl AdjustToNextNeighbour for Item {
    fn adjust_to_next(&mut self, next: &Self) {
        self.adjusted = next.number
    }
}

fn item(idx: u8) -> Item {
    Item {
        number: idx,
        adjusted: 0,
    }
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

    assert_eq!(list.first().unwrap().val, item1);
    assert_eq!(list.last().unwrap().val, item2);

    let item3 = item(3);
    list.append(item3);

    assert_eq!(list.first().unwrap().val, item1);
    assert_eq!(list.last().unwrap().val, item3);

    assert_eq!(list.next(list.first().unwrap().idx).unwrap().val, item2);
    assert_eq!(
        list.next(list.next(list.first().unwrap().idx).unwrap().idx)
            .unwrap()
            .val,
        item3
    );

    assert_eq!(list.prev(list.last().unwrap().idx).unwrap().val, item2);
    assert_eq!(
        list.prev(list.prev(list.last().unwrap().idx).unwrap().idx)
            .unwrap()
            .val,
        item1
    );

    assert_eq!(list.iter().nth(0).unwrap().val, item1);
    assert_eq!(list.iter().nth(1).unwrap().val, item2);
    assert_eq!(list.iter().nth(2).unwrap().val, item3);

    assert_eq!(list.iter().nth_back(0).unwrap().val, item3);
    assert_eq!(list.iter().nth_back(1).unwrap().val, item2);
    assert_eq!(list.iter().nth_back(2).unwrap().val, item1);

    let v = list
        .iter()
        .map_with_prev_next(|curr, prev, next| {
            curr.val.number
                + prev.map(|p| p.val.number).unwrap_or(0)
                + next.map(|n| n.val.number).unwrap_or(0)
        })
        .collect::<Vec<_>>();
    assert_eq!(v, [3, 6, 5]);

    let v = list
        .iter()
        .map(|n| n.val.number)
        .map_with_prev_next(|curr, prev, next| curr + prev.unwrap_or(0) + next.unwrap_or(0))
        .collect::<Vec<_>>();
    assert_eq!(v, [3, 6, 5]);
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
        item.val.number *= 2;
    }

    assert_eq!(list.iter().nth(0).unwrap().val.number, 2);
    assert_eq!(list.iter().nth(1).unwrap().val.number, 4);
    assert_eq!(list.iter().nth(2).unwrap().val.number, 6);
}

#[test]
fn append_at() {
    let mut list = NeighbourList::new();

    let item1 = item(10);
    let item2 = item(20);
    let item3 = item(30);

    list.append(item1);
    list.append(item2);
    list.append(item3);

    list.append_at(list.first().unwrap().idx, item(15));
    list.append_at(list.last().unwrap().idx, item(35));
    list.append_at(list.iter().nth(2).unwrap().idx, item(25));

    assert_eq!(
        list.iter().map(|n| { n.val.number }).collect::<Vec<_>>(),
        [10, 15, 20, 25, 30, 35]
    );

    // println!("{:?}", list.iter().map(|n| { n.idx }).collect::<Vec<_>>());
}
