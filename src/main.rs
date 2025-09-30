#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod data;
mod comm;
mod ui;

fn main() -> eframe::Result<()> {
    ui::start_app()
}
