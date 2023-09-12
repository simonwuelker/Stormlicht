# Contributing
I appreciate any and all contributions!
This document should act as a general guide on how to contribute your changes upstream.

## Pull Requests
Prefer multiple small pull requests to one big one. If you do end up implementing a larger feature all at once,
please open a draft PR so you can get early feedback.

## Code Style
Please make sure that your code is formatted according to the projects style guide. You can ensure this using `rustfmt` by running `cargo fmt.`

Your code should also not create warnings when compiled and checked with clippy. (check with `cargo clippy`)

### Commit messages
Commit messages should be imperative (`Do X`, `Don't allow X to Y`).
They should also include the area of code (usually the name of the library)
that was modified as a prefix.

#### Good
* `web/core: Fix off by one when parsing CSS number`
* `base: Include arch build instructions in README`

#### Bad
* `load default URL if none was provided` (**Why**: Unclear *where* changes have been made) 
* `web/core: fix bug in html parser` (**Why**: nondescriptive)

### Implementing Specifications
Please try to adhere to any relevant specifications as much as possible. 
When implementing algorithms from the spec, add a spec comment before every step, like this:
```rust
// https://html.spec.whatwg.org/multipage/parsing.html#script-data-end-tag-open-state
TokenizerState::ScriptDataEndTagOpen => {
    // Consume the next input character:
    match self.read_next() {
        Some('a'..='z' | 'A'..='Z') => {
            // Create a new end tag token, set its tag name to the empty string.
            self.current_token.create_end_tag();

            // Reconsume in the script data end tag name state.
            self.reconsume_in(TokenizerState::ScriptDataEndTagName);
        },
        _ => {
            // Emit a U+003C LESS-THAN SIGN character token and a U+002F SOLIDUS
            // character token.
            self.emit(Token::Character('<'));
            self.emit(Token::Character('/'));

            // Reconsume in the script data state.
            self.reconsume_in(TokenizerState::ScriptData);
        },
    }
},
```
If a step cannot be implemented due to missing functionality in the rest of the system, please prefix it with `FIXME:`

Sometimes its not possible or unreasonable to strictly adhere to the specifications. Deviations should *clearly* be marked as such.

When appropriate, link to the relevant sections of the specification.

### Resources
* [CSS 2](https://drafts.csswg.org/css2/)
