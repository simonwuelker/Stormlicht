#![feature(option_result_contains)]

pub mod character_reference;
pub mod dom;
pub mod parser;
pub mod tokenizer;

use parser::Parser;

const HTML: &'static str = "\
<!DOCTYPE html>
<html>
<body id=abc>
Hello World
</body>
</html>";

fn main() {
    env_logger::init();
    let document = Parser::new(&HTML).parse();
    println!("{:?}", document.borrow());
}
