use std::ops::Add;

pub fn permute_monoid_streams<'a, A: Clone + Add<Output=A>>(streams: &'a [&Fn() -> Box<Iterator<Item=A> + 'a>]) -> Box<Iterator<Item=A> + 'a> {
    match streams {
        &[] => Box::new(vec!().into_iter()),
        &[head] => head(),
        &[head, ref tail..] => Box::new(
            head().flat_map(move |a| permute_monoid_streams(tail).map(move |b| a.clone() + b))
        )
    }
}

#[test]
fn permute_numbers() {
    let numbers: &[&Fn() -> Box<Iterator<Item=u32>>] = &[
        &|| Box::new(vec!(1, 2, 3).into_iter()),
        &|| Box::new(vec!(1, 2, 3).into_iter()),
        &|| Box::new(vec!(1, 2, 3).into_iter()),
    ];
    let r: Vec<u32> = permute_monoid_streams(numbers).collect();
    assert_eq!(r, vec!(
        3, 4, 5, 4, 5, 6, 5, 6, 7,
        4, 5, 6, 5, 6, 7, 6, 7, 8,
        5, 6, 7, 6, 7, 8, 7, 8, 9
    ));
}
