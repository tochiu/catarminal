// render.rs
// transform catan data to render data :)
use substring::Substring;
use rand::Rng;
use std::io::Write;
use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

// RawRender:
// vector of 3-tuples containing details of a render
// usize  -> offset in the string buffer
// Color  -> color of the string
// String -> string containing what to store in the buffer at that offset
type RawRender = Vec<(usize, Color, String)>;

// PixelData:
// 3-tuple of render info on a "sprite" 
// usize, usize -> #rows and #cols of sprite
// u64 -> binary number that enumerates what part of the row x col canvas should be filled in starting from the top left (every #cols bits is a row)
type PixelData = (u64, usize, usize); 

// STATICS

// pixel data for each digit
static NUMS : [PixelData; 10] = [
    (0b0111010001100011000101110, 5, 5),
    (0b0110000100001000010001110, 5, 5),
    (0b1111000001011101000011111, 5, 5),
    (0b1111000001111110000111110, 5, 5),
    (0b1000110001011110000100001, 5, 5),
    (0b1111110000111100000111110, 5, 5),
    (0b0111010000111101000101110, 5, 5),
    (0b1111100010001000100010000, 5, 5),
    (0b0111010001011101000101110, 5, 5),
    (0b0111010001011110000101110, 5, 5)
];

// TODO: create canvas object
static mut canvas_width : i32 = 127;

// robber data
static ROBBER : PixelData = (0b011101111101110111111111111111, 5, 6);
static ROBBER_OFFSET : [i32; 2] = [7, -8]; // robber offset from tile offset

// digit offsets from tile offset
static ROLL_LEFT_DIGIT_OFFSET : [i32; 2] = [9, -2];
static ROLL_RIGHT_DIGIT_OFFSET : [i32; 2] = [15, -2];
static ROLL_CENTER_DIGIT_OFFSET : [i32; 2] = [12, -2];
static ROLL_RARITY_OFFSET : [i32; 2] = [14, 4];

// dimensions of land hex
static LAND_INTERNAL_WIDTH : i32 = 29;
static LAND_INTERNAL_HEIGHT : i32 = 11;

static RESOURCE_COLORS: [Color; 5] = [
    Color::Rgb(140,181,014),  // sheep
    Color::Rgb(024,152,055),  // tree
    Color::Rgb(240,185,032),  // wheat
    Color::Rgb(223,097,040),  // brick
    Color::Rgb(059,065,061)   // ore
];

// transforms a 2D coordinate into an offset in the buffer
// given a starting offset (position in string buffer) and a coordinate -> returns the correct offset
fn get_render_offset(offset: usize, coords: [i32; 2]) -> usize {
    unsafe {
        (offset as i32 + (canvas_width + 1)*coords[1] + coords[0]) as usize
    }
}

// converts pixel information into a RawRender (given an offset, a fill character and a color)
fn get_pixel_render_data(data: PixelData, offset: usize, fill: char, color: Color) -> RawRender {
    let (mut num, xsize, ysize) = data;
    let mut render_data = Vec::new();

    let mut ofx = 0; // starting offset of a chunk of filled in pixels
    let mut ofy = 0; // y position of chunk of pilled in pixels
    let mask = 1 << (xsize*ysize);

    // for each row.. 
    for _ in 0..ysize {

        /* 
        example row of 5:
          ofx   i
          v     v
        0 1 1 1 0 -> add a chunk to raw render that contains a string of 3 fill chars at the correct offset with color */

        // for each pixel in row..
        for i in 0..xsize {

            // shift num into mask pos to see if the current pixel coordinate needs to be filled
            num <<= 1;
            if num & mask == mask { // if there is a pixel here move to the next (ofx has the start location now)
               continue;
            }
            
            // if i != ofx then we have a valid chunk of pixels to fill in
            if i != ofx {
                // push a chunk of pixels to RawRender
                render_data.push((get_render_offset(offset, [ofx as i32, ofy]), color, String::from(fill).repeat(i - ofx)));
            }

            // move ofx ahead of i for next iteration since we dont have any start location to fill in pixels
            ofx = i + 1;
        }
        if ofx < xsize {
            // if ofx isn't at the end then there is one pixel chunk to push to RawRender
            render_data.push((get_render_offset(offset, [ofx as i32, ofy]), color, String::from(fill).repeat(xsize - ofx)));
        }
        ofy += 1;
        ofx = 0;
    }

    render_data
}

// TODO: create a generic get_render_layers function that returns a Vec<RawRender> and have LandRender (and future "Render" struct types) implement this

// class that contains metadata about a land tile and can convert it to a RawRender
pub struct LandRender {
    pub offset: usize, // offset of middle left part of the tile (where "[" is)
    pub color: Color,
    pub roll: u8,
    pub has_robber: bool
}

impl LandRender {

    // generate random tile data from a given string buffer offset
    pub fn from_offset(offset: usize) -> Self {
        let mut rng = rand::thread_rng();
        LandRender {
            offset: offset,
            color: RESOURCE_COLORS[rng.gen_range(0..5) as usize],
            roll: if rng.gen_range(0..2) == 0 { rng.gen_range(2..7) } else { rng.gen_range(8..13) } as u8,
            has_robber: false
        }
    }

    // get RawRender for the tile color
    pub fn get_color_render_data(&mut self) -> RawRender {
        let mut render_data = Vec::new();
        for i in (-LAND_INTERNAL_HEIGHT/2 as i32)..(LAND_INTERNAL_HEIGHT/2 + 1 as i32) {
            let offset = get_render_offset(self.offset, [i.abs(), i]);
            let amount = (LAND_INTERNAL_WIDTH - 2*i.abs()) as usize;
            render_data.push((offset, self.color, "█".repeat(amount)));
        }

        render_data
    }

    // get RawRender for the number on the tile
    pub fn get_roll_render_data(&mut self) -> RawRender {
        if self.roll == 0 {
            Vec::new()
        } else if self.roll > 9 {

            // left digit
            let mut render_data = get_pixel_render_data(
                NUMS[(self.roll / 10 % 10) as usize], 
                get_render_offset(self.offset, ROLL_LEFT_DIGIT_OFFSET),
                '█',
                Color::White, 
            );

            // right digit
            render_data.append(
                &mut get_pixel_render_data(
                    NUMS[self.roll as usize % 10], 
                    get_render_offset(self.offset, ROLL_RIGHT_DIGIT_OFFSET),
                    '█',
                    Color::White, 
                )
            );

            render_data
        } else {

            // just one digit
            get_pixel_render_data(
                NUMS[self.roll as usize], 
                get_render_offset(self.offset, ROLL_CENTER_DIGIT_OFFSET),
                '█',
                if self.roll == 8 || self.roll == 6 { Color::Rgb(255,000,000) } else { Color::White }, 
            )
        }
    }

    // get RawRender for the rarity of the roll
    pub fn get_roll_rarity_render_data(&mut self) -> RawRender {
        let mut render_data = Vec::new();
        if self.roll == 0 {
            return render_data;
        }

        let count = if self.roll > 7 { 13 - self.roll } else { self.roll - 1 } as i32;

        for i in 0..count {
            render_data.push((
                get_render_offset(self.offset, [ROLL_RARITY_OFFSET[0] - count + 2*i + 1, ROLL_RARITY_OFFSET[1]]),
                if self.roll == 8 || self.roll == 6 { Color::Rgb(255,000,000) } else { Color::White },
                String::from("█")
            ));
        }

        render_data
    }

    // get RawRender for the robber being on this tile
    pub fn get_robber_render_data(&mut self) -> RawRender {
        get_pixel_render_data(
            ROBBER, 
            get_render_offset(self.offset, ROBBER_OFFSET),
            '█',
            Color::Magenta, 
        )
    }
}

// merge the layer_up RawRender downwards to the layer_down RawRender (updates layer_down)
// IMPORTANT: IT IS ASSUMED THAT BOTH RAWRENDERS ARE SORTED
fn merge_layer_down(layer_down: &mut RawRender, layer_up: RawRender) {
    let mut i = 0;

    // for each chunk in the above layer
    for data in layer_up {

        let lbound = data.0;
        let rbound = data.0 + data.2.chars().count();

        // keep going through the below layer until we encounter a chunk that ends past the start of the above chunk
        // (it is either way past us or partially within us) -> we need to insert our above chunk here
        while i < layer_down.len() && layer_down[i].0 + layer_down[i].2.chars().count() < lbound {
            i += 1;
        }

        let mut j = i;

        // start a range and move it through all chunks below that are completely covered by the above chunk (these need to be removed)
        while j < layer_down.len() && layer_down[j].0 < rbound {
            j += 1;
        }

        let mut isplice = Vec::with_capacity(3);

        // if the chunk at i starts before we start then we need to include that part in the splice insert 
        // from when the below chunk starts up to when the above chunk starts
        if i < layer_down.len() && layer_down[i].0 < lbound {
            isplice.push((layer_down[i].0, layer_down[i].1, String::from(layer_down[i].2.substring(0, (lbound - layer_down[i].0)))));
        }

        isplice.push(data);

        // if the chunk at j - 1 starts before we end and ends after we end then we need to include that part in the splice insert
        // from when the above chunk ends up to when the below chunk ends
        if j > i && j <= layer_down.len() && layer_down[j - 1].0 < rbound && layer_down[j - 1].0 + layer_down[j - 1].2.chars().count() >= rbound {
            isplice.push((rbound, layer_down[j - 1].1, String::from(layer_down[j - 1].2.substring((rbound - layer_down[j - 1].0), layer_down[j - 1].2.chars().count()))));
        }

        // do the splice operation
        layer_down.splice(i..j, isplice);
    }
}

pub fn set_canvas_width(width: i32) {
    unsafe {
        canvas_width = width;
    }
}

// TODO: SEE get_render_players TODO -> afterwards modify this to automatically deal with an arbitrary set of render layers
// render an array of render objects (currently just LandRender) to stdout
pub fn render(canvas: &mut String, renders: &mut Vec<LandRender>) {
    // set random land tile to have the robber
    let has_robber_idx = rand::thread_rng().gen_range(0..(*renders).len()) as usize;
    renders[has_robber_idx].has_robber = true;

    // layers
    let mut render_data  : RawRender = Vec::with_capacity(5 * renders.len()); // land tile color (final layer)
    let mut render_data2 : RawRender = Vec::with_capacity(5 * renders.len()); // land tile roll
    let mut render_data3 : RawRender = Vec::with_capacity(5 * renders.len()); // land tile roll rarity
    let mut render_data4 : RawRender = Vec::with_capacity(1);                 // robber

    // append RawRenders for each layer
    for render in renders {
        render_data .append(&mut render.get_color_render_data());
        render_data2.append(&mut render.get_roll_render_data());
        render_data3.append(&mut render.get_roll_rarity_render_data());

        if render.has_robber {
            render_data4.append(&mut render.get_robber_render_data());
        }
    }

    // sort layers (render_data4 has one tile)
    render_data .sort_by(|a, b| a.0.cmp(&b.0));
    render_data2.sort_by(|a, b| a.0.cmp(&b.0));
    render_data3.sort_by(|a, b| a.0.cmp(&b.0));

    // merge render layers

    merge_layer_down(&mut render_data, render_data2);
    merge_layer_down(&mut render_data, render_data3);
    merge_layer_down(&mut render_data, render_data4);

    // translate render layer to buffer sequence

    let out_writer = BufferWriter::stdout(ColorChoice::Always);
    
    let mut buffers = Vec::with_capacity(render_data.len()); // list of buffers to print to stdout
    let mut unprinted_offset = 0; // first unprinted offset
    
    for data in render_data {
        let (offset, color, content) = data;

        // if offset isn't the same as unprinted_offset we need to store canvas[unprinted_offset..offset] (canvas chunk) in a buffer 
        if offset != unprinted_offset {
            let mut buffer = out_writer.buffer();
            buffer.set_color(ColorSpec::new().set_fg(None));
            write!(&mut buffer, "{}", canvas.substring(unprinted_offset, offset));
            buffers.push(buffer);
        }

        // store render chunk in a buffer
        let mut buffer = out_writer.buffer();
        buffer.set_color(ColorSpec::new().set_fg(Some(color)));
        write!(&mut buffer, "{}", content);
        buffers.push(buffer);

        // update unprinted offset to right after offset of render chunk
        unprinted_offset = offset + content.chars().count();
    }

    // if unprinted_offset < canvas length then we have to store canvas[unprinted_offset..canvas.chars().count()] (remaining canvas chunk) in a buffer 
    if unprinted_offset < canvas.chars().count() {
        let mut buffer = out_writer.buffer();
        buffer.set_color(ColorSpec::new().set_fg(None));
        write!(&mut buffer, "{}", canvas.substring(unprinted_offset, canvas.chars().count()));
        buffers.push(buffer);
    }

    // write buffers to stdout
    for buffer in buffers {
        out_writer.print(&buffer);
    }
}