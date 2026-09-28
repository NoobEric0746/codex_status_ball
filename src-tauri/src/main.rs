#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
  native_window::run();
}

mod native_window;
