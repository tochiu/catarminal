#[macro_use]
extern crate lazy_static;

mod render;
mod enums;
mod game;

fn main() {
    game::run().unwrap();
}
