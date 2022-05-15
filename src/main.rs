//use rand::Rng;
use colored::Colorize;      // Random
use rand::seq::SliceRandom; // Colors
use rand::Rng;              // Random Number Generator
use std::fs::File;          // Files
use std::io::prelude::*;    // Standard I/O


/////////////////////////////////////////////////////////////////////////////////////
// COLOR_TILE
// -Given a tile's [Line, Column] coordinates and the map string, draws 
//  its corresponding resource symbols onto the map to later be colored in.
//  Last Modified: 5/14/2022 
//----------------------------------------------------------------------------------
// NOTE: X-Y Coordinate for the tiles in the "map" string are defined by placing the
//       cursor to the left of the corresponding resource bracket "[".
////////////////////////////////////////////////////////////////////////////////////
fn color_tile(offset: [i32; 2], map: &mut String, resource: String) {
/*
    let possible_resources = vec![
        // Resource Symbols    RGB Values
        'S', // Sheep ------> (140, 181, 14)
        'H', // Tree  ------> (24, 152, 55)
        'A', // Wheat ------> (240, 185, 32)
        'C', // Brick ------> (223, 97, 40)
        'O', // Ore   ------> (159, 165, 161)
    ];
*/
    // Randomly choosing a resource
    let resource_symbol = resource;
    // Places the resource symbol across the particular tile
    for i in (-2 as i32)..3 {
        let idx = ((offset[0] + i - 1) * 70 + (offset[1] + i.abs() - 1)) as usize;
        let amount = (10 - 2 * i.abs()) as usize;
        let row = format!("{}",resource_symbol).repeat(amount);
        map.replace_range(idx..(idx + amount), &row);
    }
}

////////////////////////////////////////////////////////////////////////////////////////
/// ENUMERATE_TILE
/// -Given a tile's [Line,Column] coordinates and  a map string, draws in a randomly 
/// generated number on the center of the tile.
/// Last Modified: 5/14/2022
///////////////////////////////////////////////////////////////////////////////////////
fn enumerate_tile(offset: [i32; 2], map: &mut String, number: String){
    // let valid_tile_numbers = vec!["02","03","04","05","!6","!8","09","10","11","12"];
    
    //Randomly choosing a tile number
    let tile_number = number;
    let idx = ((offset[0]-1)*70 + offset[1]+3) as usize ;
    map.replace_range(idx..(idx+2),&tile_number);
}



//////////////////////////////////////////////////////////////////////////////////////
/// RENDER
/// - Takes in the map string as input and replaces all resource symbols by their
///   corresponding colored character, coloring the map, then prints it.
///   Last Modified: 5/14/2022
//////////////////////////////////////////////////////////////////////////////////////
fn render(map: &String) {
    let fill_symbol = '⋰'; //This controls what character fills the tiles
    // compiler said to make this not mutable, not sure if you have future plans for this
    let chars = map.chars();
    let  peekable_chars = chars.peekable();
    for character in peekable_chars {
        match character {
            // Color the Sheep!
            'S' => {let replacement_string = format!("{}",fill_symbol).truecolor(140, 181, 14); print!("{}", replacement_string)},
            // Color the Trees!
            'H' => {let replacement_string = format!("{}",fill_symbol).truecolor(24, 152, 55); print!("{}", replacement_string);},
            // Color the Wheat!
            'A' => {let replacement_string = format!("{}",fill_symbol).truecolor(240, 185, 32); print!("{}", replacement_string);},
            // Color the Brick!
            'C' => {let replacement_string = format!("{}",fill_symbol).truecolor(223, 97, 40); print!("{}", replacement_string);},
            // Color the Ore!
            'O' => {let replacement_string = format!("{}",fill_symbol).truecolor(159, 165, 161); print!("{}", replacement_string);},
            // Color the Ocean!
            '~' => {let replacement_string = "~".truecolor(80,174,206); print!("{}", replacement_string);},
            // Colors the 8s and 6s Red!
            '6' | '8' => {let replacement_string = format!("{}",character).truecolor(255,0,0); print!("{}", replacement_string);},
            // Does something..IDK...dont see this symbol on map
            '!' => {let replacement_string = format!("{}",'0').truecolor(255,0,0); print!("{}",replacement_string);},
            // Dont care about the rest, thy stay the same
            _ => {let new_character = format!("{}",character).truecolor(200,200,200); print!("{}",new_character);}

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

    // Vector containing all [Line, Column] coordinates of the tiles
    let tile_coords = vec![
        [10, 30],[13, 22],[13, 38],[16, 14],
        [16, 30],[16, 46],[19, 22],[19, 38],
        [22, 14],[22, 30],[22, 46],[25, 22],
        [25, 38],[28, 14],[28, 30],[28, 46],
        [31, 22],[31, 38],[34, 30],
    ];
    // Vector containing all the possible tile numbers
    let mut remaining_num = vec!["02","03","03","04","04","05","05","!6","!6","!8","!8","09","09","10","10","11","11","12"];
    // Vector containing all the possible tile resources
    let mut remaining_resource = vec!["S","S","S","S","H","H","H","H","A","A","A","A","C","C","C","O","O","O"];
    //Calls color_tile for all 19 tiles on the map.
    for i in 0..19 {
        // randomizing resource
        let rng_rsc = rand::thread_rng().gen_range(0..remaining_resource.len());
        let rsc = remaining_resource[rng_rsc];
        //creating tile resource
        color_tile(tile_coords[i], &mut map, String::from(rsc));
        // removing generated tile from vector
        remaining_resource.remove(rng_rsc);
        // randomizing numbers 
        let rng_num = rand::thread_rng().gen_range(0..remaining_num.len());
        let num = remaining_num[rng_num];
        // creating tile number
        enumerate_tile(tile_coords[i], &mut map, String::from(num));
        // removing generated tile from vector
        remaining_num.remove(rng_num);
    }
    //Rendering the final map onto the terminal
    render(&map);
}
