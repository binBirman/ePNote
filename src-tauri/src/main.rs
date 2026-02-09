// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;
mod util;

use app_lib::error::AppError;
use std::path::PathBuf;

fn main() {}
