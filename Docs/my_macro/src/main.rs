extern crate my_macros;
use my_macros::answer_fn;

answer_fn!();

fn main() {
    print!("This is my answer: {}\n", answer());
}
