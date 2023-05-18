# Browser
(The name is a work in progress)

## Design goals
I strive for Correctness, Performance and Safety, in that order.

This is also a "for fun" project, so I do try and implement as much of the functionality as possible without relying on third party crates. (currently, we only need [glazier](https://github.com/linebender/glazier) and [softbuffer](https://github.com/rust-windowing/softbuffer) for cross-platform window management and `syn`/`quote`/`proc-macro2` during compilation)

## Build Instructions
Simply run
```
sudo apt install build-essential python3
```
and
```
python3 setup.py
```
to download all required files (mostly fonts) and then
```
cargo r
```
to compile and run the browser.


## Credits
This project is heavily inspired by [Andreas Kling](https://github.com/awesomekling)/[the Ladybird Browser](https://awesomekling.github.io/Ladybird-a-new-cross-platform-browser-project/)
