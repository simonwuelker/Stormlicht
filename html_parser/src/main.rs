#![feature(option_result_contains)]

pub mod character_reference;
pub mod dom;
pub mod parser;
pub mod tokenizer;

use parser::Parser;

const HTML: &'static str = "\
<html>
<body id=abc>
Hello World
</body>
</html>";

fn main() {
    Parser::new(&HTML).parse();
}
