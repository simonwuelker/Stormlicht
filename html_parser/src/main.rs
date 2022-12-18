#![feature(option_result_contains)]

pub mod character_reference;
pub mod dom;
pub mod parser;
pub mod tokenizer;

use parser::Parser;

const HTML: &'static str = "\
<!DOCTYPE html>
<html attribute=value>
<body id=abc>
Hello World
<!-- Comment -->
</body>
</html>";

fn main() {
    let document = Parser::new(&HTML).parse();
    println!("{:?}", document.borrow());
}
