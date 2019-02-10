extern crate psm;

fn main() {
    println!("Stack is {:?} and is at {:p} currently",
             psm::StackDirection::new(), psm::stack_pointer());
}

#[test]
fn run_example() {
    main();
}
