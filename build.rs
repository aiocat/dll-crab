// Copyright (c) 2022 aiocat
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use winres::WindowsResource;

fn main() {
    let mut res = WindowsResource::new();
    res.set_icon(".\\assets\\dll-crab.ico")
        .set("InternalName", "DLL Crab")
        .set_language(0x0409)
        .set("CompanyName", "Aiocat");

    res.compile().unwrap();
}
