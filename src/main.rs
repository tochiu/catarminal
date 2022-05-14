//use rand::Rng;
use colored::Colorize;      // Random
use rand::seq::SliceRandom; // Colors
use std::fs::File;          // Files
use std::io::prelude::*;    // Standard I/O


///////////////////////////////////////////////////////////////////////
// COLOR_TILE
// -Given a tile's [Line, Column] coordinates, draws its corresponding
//  resource symbols onto the map to later be colored in.
//  Last Modified:5/14/2022 
///////////////////////////////////////////////////////////////////////
fn color_tile(offset: [i32; 2], map: &mut String) {
    let possible_resources = vec![
        // Resource Symbols    RGB Values
        'S', // Sheep ------> 
        'T', // Tree  ------>
        'W', // Wheat ------>
        'B', // Brick ------>
        'O', // Ore   ------>
    ];
    let mut rng = rand::thread_rng();
    // NOTE: X-Y Coordinate for the tiles in the "map" string are defined by placing the
    //       cursor to the left of the corresponding resource bracket "[".

    // Randomly choosing a resource
    let resource_symbol = possible_resources.choose(&mut rng).unwrap();
    // Places the resource symbol across the particular tile
    for i in (-2 as i32)..3 {
        let idx = ((offset[0] + i - 1) * 70 + (offset[1] + i.abs() - 1)) as usize;
        let amount = (10 - 2 * i.abs()) as usize;
        let row = format!("{}",resource_symbol).repeat(amount);
        map.replace_range(idx..(idx + amount), &row);
    }
}


//////////////////////////////////////////////////////////////////////////////////////
/// RENDER
/// - Takes in the map string as input and replaces all resource symbols by their
///   corresponding colored character, coloring the map, then prints it.
///   Last Modified: 5/14/2022
//////////////////////////////////////////////////////////////////////////////////////
fn render(map: &String) {
    let fill_symbol = '⋰'; //This controls what character fills the tiles
    for character in map.chars() {
        if character == 'S' { //Color the Sheep!
            let replacement_string = format!("{}",fill_symbol).truecolor(140, 181, 14);
            print!("{}", replacement_string);}

            //TODO: Replace these with a match statement

        if character == 'T' { //Color the Trees!
            let replacement_string = format!("{}",fill_symbol).truecolor(24, 152, 55);
            print!("{}", replacement_string);}

        if character == 'W' { //Color the Wheat!
            let replacement_string = format!("{}",fill_symbol).truecolor(240, 185, 32);
            print!("{}", replacement_string);}

        if character == 'B' { //Color the Bricks!
            let replacement_string = format!("{}",fill_symbol).truecolor(223, 97, 40);
            print!("{}", replacement_string);}

        if character == 'O' { //Color the Ore!
            let replacement_string = format!("{}",fill_symbol).truecolor(159, 165, 161);
            print!("{}", replacement_string);}
            
        if character != 'O' && character != 'B' && character != 'W' && character != 'T' && character != 'S' {
            print!("{}", character);
        }
    }
}
//////////////////////////////////////////////////////////////////////////////////

fn main() {
    // Maps the num of each tile to the ranges in the string its area covers

    // Opening map.txt
    let mut file = File::open("map.txt").expect("Cannot open the file");
    // Creating a new empty string
    let mut map = String::new();
    // Reading the map of map.txt into the new string
    file.read_to_string(&mut map).expect("Cannot read the file");

    //Vector containing all [Line, Column] coordinates of the tiles
    let tile_coords = vec![
        [10, 30],[13, 22],[13, 38],[16, 14],
        [16, 30],[16, 46],[19, 22],[19, 38],
        [22, 14],[22, 30],[22, 46],[25, 22],
        [25, 38],[28, 14],[28, 30],[28, 46],
        [31, 22],[31, 38],[34, 30],
    ];
    //Calls color_tile for all 19 tiles on the map.
    for i in 0..19 {
        color_tile(tile_coords[i], &mut map);
    }
    //Rendering the final map onto the terminal
    render(&map);
}
