use std::fs::File;
use std::io::prelude::*;

mod render;

fn main() {
    let mut file = File::open("map.txt").expect("Cannot open the file");
    let mut map_str = String::new();
    file.read_to_string(&mut map_str).expect("Cannot read the file");

    let mut renders = Vec::new();

    for (i, c) in map_str.chars().enumerate() {
        if c == '[' {
            renders.push(render::LandRender::from_offset(i));
        }
    }

    // test
    render::render(&mut map_str, &mut renders);
}
