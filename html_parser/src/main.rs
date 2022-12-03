#![feature(option_result_contains)]

pub mod character_reference;
pub mod dom;
pub mod parser;
pub mod tokenizer;

use parser::Parser;
use tokenizer::Tokenizer;

const HTML: &'static str = "\
<html>
<body id=abc>
Hello World
</body>
</html>";

fn main() {
    let mut tokenizer = Tokenizer::new(&HTML);
    let mut parser = Parser::new();
    while let Some(t) = tokenizer.next() {
        println!("emitted {:?}", t);
        parser.consume(t);
    }
}
