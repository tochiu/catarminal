use std::fs::File;
use std::io::prelude::*;

mod render;

fn main() {
    let mut file = File::open("map.txt").expect("Cannot open the file");
    let mut map_str = String::new();
    file.read_to_string(&mut map_str).expect("Cannot read the file");

    let mut line_len = 0;

    let mut offsets : Vec<usize> = Vec::new();

    for (i, c) in map_str.chars().enumerate() {
        if c == '[' {
            offsets.push(i);
        } else if c == '\n' && line_len == 0 {
            line_len = i;
            render::set_canvas_width(line_len as i32);
        }
    }

    // test
    render::render(
        &mut map_str, 
        &mut offsets.iter().map(|&offset| render::LandRender::from_offset(offset)).collect()
    );
}
