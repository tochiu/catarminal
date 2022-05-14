use std::fs::File;
use std::io::prelude::*;

mod render;

fn main() {
    let mut file = File::open("map.txt").expect("Cannot open the file");
    let mut map_str = String::new();
    file.read_to_string(&mut map_str).expect("Cannot read the file");

    let mut renders = Vec::new();
    let mut line_len = 0;

    for (i, c) in map_str.chars().enumerate() {
        if c == '[' {
            renders.push(render::LandRender::from_offset(i));
        } else if c == '\n' && line_len == 0 {
            line_len = i;

            // this is fine because we find a \n before we find any other special characters
            render::set_canvas_width(line_len as i32); 

            // in the future we will first create the canvas object then augmenting it with renders
        }
    }

    // test
    render::render(&mut map_str, &mut renders);
}
