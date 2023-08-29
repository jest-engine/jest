use jest;

mod common;

#[test]
fn it_adds_two() {
    common::setup();
    assert_eq!(4, jest::add_two(2));
}