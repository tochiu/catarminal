//use rand::Rng;
use colored::Colorize;
use rand::seq::SliceRandom;
use std::fs::File;
use std::io::prelude::*;

// Generates the starting map and returns the string containing the map.

fn color_tile(offset: [i32; 2], map: &mut String) {

    let rgb_values = vec![
        [140,181,14], // Sheep Color
        [24,152,55],  // Tree Color
        [240,185,32], // Wheat Color
        [223,97,40],  // Brick Color
        [159,165,161] // Ore Color
        ];

    let mut rng = rand::thread_rng();
    // NOTE: X-Y Coordinate for the tiles are defined by placing the curson to the
    //       left of the resource bracket "["
    for i in (-2 as i32)..3 {
        let idx = ((offset[0] + i - 1 )*70 + (offset[1] + i.abs() - 1)) as usize;
        let amount = (10 - 2*i.abs()) as usize;
        let random_resource_choice = rgb_values.choose(&mut rng).unwrap();
        let colored_row = ("#".repeat(amount)).truecolor(
            random_resource_choice[0], // R
            random_resource_choice[1], // G
            random_resource_choice[2]  // B
        );
        println!("{}", colored_row);
        map.replace_range(idx..(idx + amount), &colored_row);
    }
}


fn main() {


    // Maps the num of each tile to the ranges in the string its area covers

    // Opening map.txt
    let mut file = File::open("map.txt").expect("Cannot open the file");
    // Creating a new empty string
    let mut map = String::new();
    // Reading the map of map.txt into the new string
    file.read_to_string(&mut map).expect("Cannot read the file");

    color_tile([28, 14], &mut map);

    println!("{}", map);

}
