# Specifications
The web and its surrounding architecture is mostly well defined in the form of specifications.
Browser engines follow this behaviour as closely as possible to ensure identical behaviour.

Since specifications can be hard to navigate, especially as a beginner, this page aims to assist
in finding the right specification and implementing it.

## HTML
HTML is mostly specified in [html.spec.whatwg.org](https://html.spec.whatwg.org/).

## CSS
The behaviour and syntax of CSS is scattered across many different specifications.
The "core" is the [CSS2](https://drafts.csswg.org/css2/) spec. It covers the high-level
parsing and layout process. More detailed information can be found in specifications like 
[CSS-Syntax-3](https://drafts.csswg.org/css-syntax-3/) or [Selectors-3](https://drafts.csswg.org/selectors-3/).

A full list of stabilized specs can be found [here](https://www.w3.org/Style/CSS/specs.en.html). Many
relevant specifications are drafts, ther list is [here](https://drafts.csswg.org/). It is okay (and usually
preferred in the case of Stormlicht) to refer to a draft version of a specification.

## JavaScript (ECMAScript)
Javascript (or formally ECMAScript) is entirely specified in [262.ecma-international.org](https://262.ecma-international.org/).
The official test suite is [Test262](https://github.com/tc39/test262).
A (unofficial) testrunner can be used in your browser from [bakkot.github.io/test262-web-runner](https://bakkot.github.io/test262-web-runner/).

## Implementing specified behaviour in Stormlicht
Behaviour that is formally defined by a specification should **always** be marked as such, with a link
to the relevant section. This makes it easier to verify the correctness and to update the code in case
the specifications change. This usually looks like this (example from `treebuilding/parser.rs`):

```rust,ignore
/// <https://html.spec.whatwg.org/multipage/parsing.html#close-a-p-element>
fn close_p_element(&mut self) {
    // 1. Generate implied end tags, except for p elements.
    self.generate_implied_end_tags_excluding(Some(static_interned!("p")));

    // 2. If the current node is not a p element,
    if !self.current_node().is_a::<HtmlParagraphElement>() {
        // then this is a parse error.
    }

    // 3. Pop elements from the stack of open elements until 
    //    a p element has been popped from the stack.
    self.pop_from_open_elements_until(|node| node.is_a::<HtmlParagraphElement>());
}
```
Notice the important parts here: 
* The function includes a link to its specification in `rustdoc` format
* The individual steps are included as comments next to the code
* The function does not skip the empty `if` branch (parse errors are optional and are not reported at the moment)

If you deviate from the spec (for performance reasons, or because the spec is being silly) you should
mark your code clearly as such, with an explanatory comment.

If part of a specification cannot be implemented because the engine is missing some required parts, you should
prefix the spec steps with `FIXME`, like this:
```rust,ignore
// 7. FIXME: If last node is furthest block, then move the aforementioned bookmark 
//    to be immediately after the new node in the list of active formatting elements.
```