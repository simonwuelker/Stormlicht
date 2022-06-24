#![feature(option_result_contains)]

mod dom;
mod html_tokenizer;
mod character_reference;

use html_tokenizer::*;

const HTML: &'static str = "
<html>
<body id=abc>
Hello World
</body>
</html>";

fn main() {
    let mut tokenizer = Tokenizer::new(&HTML);
    while tokenizer.run {
        tokenizer.step();
    }
    // let mut root = DomNode::new("html".to_string());
    // root.append(DomNode::text("Hello World".to_string()));
    // root.append(DomNode::new("body".to_string()));
}
