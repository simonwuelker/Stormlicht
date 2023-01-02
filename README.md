# Browser
(The name is a work in progress)

## Code
Here are some of the cool things built (from scratch) for this browser:

| Path | Description | Completeness | Resources |
| ---- | ----------- | ------------ | --------- |
| [`/web/dns`](https://github.com/Wuelle/browser/tree/main/web/dns) | DNS-Resolver | Usable | [RFC 1034](https://www.rfc-editor.org/rfc/rfc1034) |
| [`/web/url`](https://github.com/Wuelle/browser/tree/main/web/url) | URL-Parser | Usable |  [url.spec.whatwg.org](https://url.spec.whatwg.org/) |
| [`/web/http`](https://github.com/Wuelle/browser/tree/main/web/http) | HTTP/1.1 Client | Usable |  [RFC 9112](https://datatracker.ietf.org/doc/html/rfc9112) |
| [`/web/html`](https://github.com/Wuelle/browser/tree/main/web/http) | HTML Parser | Tokenizer complete, Tree builder is WIP | [html.spec.whatwg.org](https://html.spec.whatwg.org/) |
| [`/font_rasterizer`](https://github.com/Wuelle/browser/tree/main/font_rasterizer) | `.ttf` Font parser & rasterizer | Usable | |
| [`/parser_combinators`](https://github.com/Wuelle/browser/tree/main/parser_combinators) | PEG Library | Usable | |

## Building
Simply run
```
./download.sh
```
to download all required files and then
```
cargo r
```
to compile and run the browser.


## Credits
This project is heavily inspired by [Andreas Kling](https://github.com/awesomekling)/[the LadyBird Browser](https://github.com/SerenityOS/ladybird)
