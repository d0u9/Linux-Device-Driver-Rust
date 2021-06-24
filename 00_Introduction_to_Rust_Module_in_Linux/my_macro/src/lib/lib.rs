extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro]
pub fn answer_fn(_item: TokenStream) -> TokenStream {
    println!("This line is printed in macro");
    "
        fn answer() -> u32 {
            let k = 1234;
            print!(\"The answer is: {}\n\", k);
            k
        }
    ".parse().unwrap()
}

