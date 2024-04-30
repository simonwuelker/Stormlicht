[![rustfmt](https://github.com/Wuelle/Stormlicht/actions/workflows/rustfmt.yaml/badge.svg)](https://github.com/Wuelle/Stormlicht/actions/workflows/rustfmt.yaml)

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://user-images.githubusercontent.com/58120269/241563717-f73e2144-9101-4d3f-b7d2-ed2459e5d8e0.svg" width="200px">
  <source media="(prefers-color-scheme: light)" srcset="https://user-images.githubusercontent.com/58120269/241563716-fde2bdf7-7ec4-48ee-928e-fa5b6a2625f2.svg" width="200px">
  <img alt="The outline of a hurricane lamp." src="https://user-images.githubusercontent.com/58120269/241563716-fde2bdf7-7ec4-48ee-928e-fa5b6a2625f2.svg" width="200px" align="right">
</picture>


# 1. Stormlicht
Stormlicht is an experimental browser engine written from scratch.<br>
If you want to follow the development, you can visit [#stormlicht:matrix.org](https://app.element.io/#/room/#stormlicht:matrix.org)

- [1. Stormlicht](#1-stormlicht)
  - [1.1. Design goals](#11-design-goals)
  - [1.2. Build Instructions](#12-build-instructions)
  - [1.3. Development](#13-development)
    - [1.3.1 Logging](#131-logging)
    - [1.3.2 Backtraces](#132-backtraces)
    - [1.3.3 Running with Miri](#133-running-with-miri)
  - [1.4. Why is there no GUI?](#14-why-is-there-no-gui)
  - [1.5. Credits](#15-credits)


## 1.1. Design goals
I strive for Correctness, Performance and Safety, in that order.

This is also a "for fun" project, so I do try and implement as much of the functionality as possible without relying on third party crates.
The engine implements its own font rendering, http stack and image parsing, among other things.

Despite being a "hobby project", Stormlicht is making good progress and has already helped fix bugs in web specifications and implemented [WPT](https://github.com/web-platform-tests/wpt) tests.

| Rendering the [Acid1](https://www.w3.org/Style/CSS/Test/CSS1/current/test5526c.htm) test            | Reference Rendering (Mozilla Firefox 122.0)                                                         |
| --------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------- |
| ![image](https://github.com/Wuelle/Stormlicht/assets/58120269/7c669bbb-9d60-44b4-921b-40ec99c655c4) | ![image](https://github.com/Wuelle/Stormlicht/assets/58120269/75502ae0-e363-40a5-9fae-4049b04a79fd) |




## 1.2. Build Instructions
Stormlicht uses the latest nightly compiler version.
First, install the rust compiler[^1] , then switch to nightly using 
```console
rustup default nightly
```

After installing rust, run the following commands to start the python build script
```console
pip install -r requirements.txt

./stormlicht.py run
```


## 1.3. Development
### 1.3.1 Logging
During debugging, you can use the `RUST_LOG` environment variable
to set the log level. Refer to the documentation of [env-logger](https://docs.rs/env_logger/latest/env_logger/) for more complex log syntax.

For example:
```console
# Log "debug" and above
RUST_LOG=debug ./stormlicht.py run
```
Available levels are `trace`, `debug`, `info`, `warn` and `error`, in ascending order.

The default log level is `info`

### 1.3.2 Backtraces
Set `RUST_BACKTRACE=1` to enable backtraces in case of a panic.

### 1.3.3 Running with Miri
Testing the browser inside [Miri](https://github.com/rust-lang/miri) can help detect instances of undefined behaviour. To use miri, execute the following:
```
rustup toolchain install nightly --component miri
rustup override set nightly

./stormlicht.py run --miri
./stormlicht.py test --miri
```

## 1.4. Why is there no GUI?
I would love to have a GUI! But actually writing one in rust is *hard*[^2], mostly due to the lack of OOP and the borrowchecker.
There are currently very smart people working to solve these issues[^3], but I am not one of them. If you want to try implementing a  GUI, please do!

But there is another reason: A browser engine is really just a highly complex framework for implementing user interfaces. If I procrastinate on implementing a GUI library long enough, i can just write it in HTML!

## 1.5. Credits
This project is inspired by [Andreas Kling](https://github.com/awesomekling)/[the Ladybird Browser](https://awesomekling.github.io/Ladybird-a-new-cross-platform-browser-project/)

Mozilla's [Servo](https://servo.org/) and [WebKit](https://github.com/WebKit/WebKit) provided some good ideas.

[^1]: https://www.rust-lang.org/tools/install
[^2]: https://www.areweguiyet.com/
[^3]: https://raphlinus.github.io/rust/gui/2022/05/07/ui-architecture.html
