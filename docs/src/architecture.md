# Architecture
Stormlicht is made up of a number of crates. These crates are found in the `crates/` directory.
Unlike most other rust crates, these are not published on [crates.io](https://crates.io/), since they're
not intended for public use. This allows for tighter integration and makes breaking changes easier to perform.

## The `web` crate
`web` is at the core of Stormlicht. It takes care of parsing HTML and CSS code, building the DOM, performing layout
and converting the page to a series of draw calls.