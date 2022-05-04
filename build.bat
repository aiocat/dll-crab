: Copyright (c) 2022 aiocat
: 
: This software is released under the MIT License.
: https://opensource.org/licenses/MIT

: remove old files if folder exists
if exist .\build (
    del .\build\dll-crab-gnu.exe
    del .\build\dll-crab-msvc.exe
    del .\build\dll-crab-gnu.checksum.txt
    del .\build\dll-crab-msvc.checksum.txt
) else (
    mkdir .\build
)

: compile for windows-gnu
rustup override set stable-x86_64-pc-windows-gnu
cargo build --release
move .\target\release\dll-crab.exe .\build\dll-crab-gnu.exe
: generate sha256 hash
certutil -hashfile ".\build\dll-crab-gnu.exe" SHA256 >> .\build\dll-crab-gnu.checksum.txt

: compile for windows-msvc
rustup override set stable-x86_64-pc-windows-msvc
cargo build --release
move .\target\release\dll-crab.exe .\build\dll-crab-msvc.exe
: generate sha256 hash
certutil -hashfile ".\build\dll-crab-msvc.exe" SHA256 >> .\build\dll-crab-msvc.checksum.txt

: clear override
rustup override unset