#[macro_use]
extern crate lazy_static;
extern crate log;

mod render;
mod enums;
mod game;

fn main() -> Result<(), std::io::Error> {
    tui_logger::init_logger(log::LevelFilter::Trace).unwrap();
    tui_logger::set_default_level(log::LevelFilter::Trace);
    game::start(std::env::args().nth(1) == Some(String::from("log")))
}
