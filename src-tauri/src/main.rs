// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// 将 lib.rs 声明为一个模块
mod lib;

fn main() {
  // 调用 lib 模块中的 run 函数
  lib::run();
}
// 