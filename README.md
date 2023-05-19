# 1. Browser
(The name is a work-in-progress)

- [1. Browser](#1-browser)
  - [1.1. Design goals](#11-design-goals)
  - [1.2. Build Instructions](#12-build-instructions)
    - [1.2.1. Debian/Ubuntu](#121-debianubuntu)
  - [1.3. Development](#13-development)
  - [1.4. Why is there no GUI?](#14-why-is-there-no-gui)
  - [1.5. Credits](#15-credits)


## 1.1. Design goals
I strive for Correctness, Performance and Safety, in that order.

This is also a "for fun" project, so I do try and implement as much of the functionality as possible without relying on third party crates. (currently, we only need [glazier](https://github.com/linebender/glazier) and [softbuffer](https://github.com/rust-windowing/softbuffer) for cross-platform window management, `log` and `env-logger` for logging as well as `syn`/`quote`/`proc-macro2` during compilation)

## 1.2. Build Instructions
### 1.2.1. Debian/Ubuntu
```sh
# Install required dependencies
sudo apt install build-essential python3

# Download required files (fonts etc..)
python3 setup.py

# Run the browser
cargo r -- <url>
```

## 1.3. Development
During debugging, you can use the `RUST_LOG` environment variable
to set the log level. Refer to the documentation of [env-logger](https://docs.rs/env_logger/latest/env_logger/) for more complex log syntax.

For example:
```sh
# Log "info" and above
RUST_LOG=info cargo r -- <url>
```
Available levels are `trace`, `debug`, `info`, `warn` and `error`, in ascending order.

The default log level is `warn`

## 1.4. Why is there no GUI?
I would love to have a GUI! But actually writing one in rust is *hard*, mostly due to the lack of OOP and the borrowchecker. (See also: [areweguiyet.com](http://www.areweguiyet.com/))
There are currently very smart people [working to solve these issues](https://raphlinus.github.io/rust/gui/2022/05/07/ui-architecture.html), but I am not one of them. If you want to try implementing a  GUI, please do!

But there is another reason: A browser engine is really just a highly complex framework for implementing user interfaces. If I procrastinate on implementing a GUI library long enough, i can just write it in HTML!


## 1.5. Credits
This project is inspired by [Andreas Kling](https://github.com/awesomekling)/[the Ladybird Browser](https://awesomekling.github.io/Ladybird-a-new-cross-platform-browser-project/)

Mozilla's [Servo](https://servo.org/) provided some good ideas.
