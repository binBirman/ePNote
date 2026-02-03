// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;

use std::path::PathBuf;

fn main() {
    let data_root = PathBuf::from("./DataRoot");

    std::fs::create_dir_all(&data_root).unwrap();

    let conn = db::init_db(&data_root).unwrap();

    println!("DB initialized: {:?}", conn);
}
