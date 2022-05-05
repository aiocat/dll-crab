: Copyright (c) 2022 aiocat
: 
: This software is released under the MIT License.
: https://opensource.org/licenses/MIT

: close echo
@echo off

: format code before compile
cargo fmt

: remove old files if folder exists
if exist .\build (
    rmdir /Q /S .\build 
) else (
    mkdir .\build
)

: compile for both gnu and msvc
for %%t in (gnu msvc) do (
    : set override and compile
    rustup override set stable-x86_64-pc-windows-%%t
    cargo build --release

    : create gnu build folder
    mkdir .\build\dll-crab-%%t

    : move file to folder
    move .\target\release\dll-crab.exe .\build\dll-crab-%%t\dll-crab.exe

    : copy crab icon, license and readme file
    echo f | xcopy /f /y .\assets\dll-crab.ico .\build\dll-crab-%%t\dll-crab.ico
    echo f | xcopy /f /y .\README.md .\build\dll-crab-%%t\README.md
    echo f | xcopy /f /y .\LICENSE .\build\dll-crab-%%t\LICENSE

    : into build folder
    cd .\build

    : generate sha256 hash
    certutil -hashfile ".\dll-crab-%%t\dll-crab.exe" SHA256 >> .\dll-crab-%%t\checksum.txt

    : create zip
    tar.exe -a -cf dll-crab-%%t.zip dll-crab-%%t

    : delete folder if argument given
    if "%1" == "clean" (
        rmdir /Q /S .\dll-crab-%%t
    )

    : move to source folder
    cd ..
)

: clear override
rustup override unset