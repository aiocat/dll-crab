// Copyright (c) 2022 aiocat
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use rand::{distributions::Alphanumeric, Rng};
use std::fs::{create_dir, read, write};
use std::path::PathBuf;

// Read DLL data and write it to new random-named file
pub fn spoof_dll(path: String) -> String {
    let new_name = format!("{}.dll", random_name(10));
    let mut data = read(&path).unwrap();

    // change hash
    (0..random_int(10, 100)).for_each(|_| data.push(0x0));

    // change path
    let mut new_path = PathBuf::from(&path);
    new_path.pop();
    new_path.push(".dcspf");

    if !new_path.is_dir() {
        create_dir(&new_path).unwrap();
    }

    // write edited data
    new_path.push(new_name);
    write(&new_path, data).unwrap();

    new_path.as_os_str().to_str().unwrap().to_string()
}

fn random_name(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

fn random_int(min: i8, max: i8) -> i8 {
    let mut rng = rand::thread_rng();
    rng.gen_range(min..max)
}
