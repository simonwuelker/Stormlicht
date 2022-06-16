#![feature(option_result_contains)]

mod dom;
mod html_tokenizer;
mod error;

use dom::*;
use html_tokenizer::*;

const HTML: &'static str = "
<html>
<body>
Hello World
</body>
</html>";


fn main() {
    let mut tokenizer = Tokenizer::new(&HTML);
    // let mut root = DomNode::new("html".to_string());
    // root.append(DomNode::text("Hello World".to_string()));
    // root.append(DomNode::new("body".to_string()));

}
