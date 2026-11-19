// Évite la fenêtre de console sous Windows en version release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    cineplan_desktop_lib::run()
}
