#![feature(option_result_contains)]

mod tokenizer;
mod parser;
mod character_reference;

use tokenizer::*;

const HTML: &'static str = "
<html>
<body id=abc>
Hello World
</body>
</html>";

fn main() {
    let mut tokenizer = Tokenizer::new(&HTML);
    while let Some(t) = tokenizer.next() {
        println!("emitted {:?}", t);
    }
}
