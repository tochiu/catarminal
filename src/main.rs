//use rand::Rng;
use colored::Colorize;
use rand::seq::SliceRandom;
use std::fs::File;
use std::io::prelude::*;

// Generates the starting map and returns the string containing the map.

fn color_tile2(offset: [i32; 2], map: &mut colored::ColoredString) {

    let rgb_values = vec![
        [140,181,14], // Sheep Color
        [24,152,55],  // Tree Color
        [240,185,32], // Wheat Color
        [223,97,40],  // Brick Color
        [159,165,161] // Ore Color
        ];

    let mut rng = rand::thread_rng();

    for i in (-2 as i32)..3 {
        let idx = ((offset[0] + i - 1 )*69 + (offset[1] + i.abs() - 2)) as usize;
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

// fn color_tile(i: usize,  map: &mut String, color: &colored::ColoredString) {
//     // Tile coords where the i'th element is the value corresponding to its upper-left most tile area character
//     let tile_coords = vec![
//         514, 713, 729, 912, 928, 944, 1127, 1143, 1326, 1342, 1358, 1541, 1557, 1740, 1756, 1772,
//         1955, 1971, 2170,
//     ]; 
    // Selecting a random resource to color the tile
    
    
    //let coord = tile_coords2[i];

    // let color = &color[..];
    // let row1 = "######"
    // let row2 = "########"
    // let row3 = "##########"
    // map.replace_range(tile_coords[i]..(tile_coords[i]) + 6,&row1 );
    // map.replace_range((tile_coords[i]+68)..((tile_coords[i]) + 8 + 68),&row2 );
    // map.replace_range((tile_coords[i]+136)..((tile_coords[i]) + 10 + 136), &row3);
    // map.replace_range((tile_coords[i]+206)..((tile_coords[i]) + 8+206),&row2);
    // map.replace_range((tile_coords[i]+276)..((tile_coords[i]) + 6+276), &row1);
// }

fn main() {
    // let resource_colors = vec![
    //     "#".truecolor(140, 181, 14),  // Sheep Color
    //     "#".truecolor(24, 152, 55),   // Trees Color
    //     "#".truecolor(240, 185, 32),  // Wheat Color
    //     "#".truecolor(223, 97, 40),   // Brick Color
    //     "#".truecolor(159, 165, 161), // Ore Color
    // ];

    // let rgb_values = vec![
    //     [140,181,14], // Sheep Color
    //     [24,152,55],  // Tree Color
    //     [240,185,32], // Wheat Color
    //     [223,97,40],  // Brick Color
    //     [159,165,161] // Ore Color
    //     ];


    // Maps the num of each tile to the ranges in the string its area covers

    // Opening map.txt
    let mut file = File::open("map.txt").expect("Cannot open the file");
    // Creating a new empty string
    let mut map = String::new();
    // Reading the map of map.txt into the new string
    file.read_to_string(&mut map).expect("Cannot read the file");
    // Looping over all 19 tiles and filling them with their resource color
    // for i in (0 as usize)..19 {
    //     //Choosing random resource color from array of resource_colors to be placed into a tile
    //     let mut rng = rand::thread_rng();
    //     //let color = resource_colors.choose(&mut rng).unwrap();

    //     //color_tile(i, &mut map, color);


    //     if i == 18 {
    //         println!("{}", map);
    //     }
       
    // }

    // let tile_coords2 = [[10, 31]];
    // let inits = [10, 2];
    
    // let 
    // {
        
    // }

    // let row = 1;
    // let col = 0;
    
    // for c in map.chars() {
    //     if c is 
    // }

    color_tile2([22, 31], &mut map);

    println!("{}", map);
    println!("\x1b[93mError\x1b[0m");
}
