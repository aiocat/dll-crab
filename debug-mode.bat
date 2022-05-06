: Copyright (c) 2022 aiocat
: 
: This software is released under the MIT License.
: https://opensource.org/licenses/MIT

: close echo
@echo off

: infinite loop
:loop
cls
cargo fmt
cargo clippy
cargo build
cls
cd .\target\debug
.\dll-crab.exe
cd ..
cd ..
goto loop