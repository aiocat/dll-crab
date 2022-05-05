<!--
 Copyright (c) 2022 aiocat

 This software is released under the MIT License.
 https://opensource.org/licenses/MIT
-->

<div align="center">

![Logo](./assets/dll-crab.png)

# DLL Crab

Rusty DLL Injector with GUI

[![Build status](https://ci.appveyor.com/api/projects/status/h6fpyexoryiddtv7?svg=true)](https://ci.appveyor.com/project/aiocat/dll-crab)
[![Unsafe](https://img.shields.io/badge/unsafe-%E2%9C%94-C901DD.svg)](https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html)

## Screenshot

![Screenshot 1](./assets/screenshot_1.png)
![Screenshot 2](./assets/screenshot_2.png)

</div>

## Why?

Because I can't find a GUI DLL Injector that written in Rust. And i wrote it myself!

## Methods

- CreateRemoteThread
- RtlCreateUserThread
- QueueUserAPC
- NtCreateThreadEx

## Download

You can download latest release from [here](https://github.com/aiocat/dll-crab/releases/latest).

## DCB (DLL Crab Builder)

`dcb.bat` is a batch script for building dll crab. If you have Rust, you can use this script to build dll-crab automatically for both `gnu` and `msvc` toolchains.

- `.\dcb.bat`: Default building
- `.\dcb.bat clean`: Only ZIP files _(used in github releases)_

And check `.\build` folder after script is finished.

## Technologies

- Rust for Everything
- `egui` for GUI

## Contributing

All pull-requests and issues are welcome. Just make sure you got a brain.

If you got an error, Please open an issue at [here](https://github.com/aiocat/dll-crab/issues).

## Building

### Pre-Requests

- Rust compiler and Cargo must be installed to your computer

### Progress

- Clone the repo (`git clone git@github.com:aiocat/dll-crab.git`)
- Move into folder (`cd dll-crab`)
- Run cargo build (`cargo build --release`)

## License

DLL Crab is distributed under MIT license. for more information:

- https://raw.githubusercontent.com/aiocat/dll-crab/main/LICENSE
