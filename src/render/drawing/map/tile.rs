use super::{super::{
    shape::*,
    super::{
        draw::*,
        space::*,
        screen::*,
        anim::*
    }
}, MAP_SAND_COLOR};

use crate::enums;

use tui::{
    style::{Color, Style},
    buffer::{Cell, Buffer}, layout::Rect
};

use std::time::Instant;

const DIGITS: [BitShape128; 10] = [
    BitShape128::new(0b0111010001100011000101110, Size2D::new(5, 5)),
    BitShape128::new(0b0110000100001000010001110, Size2D::new(5, 5)),
    BitShape128::new(0b1111000001011101000011111, Size2D::new(5, 5)),
    BitShape128::new(0b1111000001111110000111110, Size2D::new(5, 5)),
    BitShape128::new(0b1000110001011110000100001, Size2D::new(5, 5)),
    BitShape128::new(0b1111110000111100000111110, Size2D::new(5, 5)),
    BitShape128::new(0b0111010000111101000101110, Size2D::new(5, 5)),
    BitShape128::new(0b1111100010001000100010000, Size2D::new(5, 5)),
    BitShape128::new(0b0111010001011101000101110, Size2D::new(5, 5)),
    BitShape128::new(0b0111010001011110000101110, Size2D::new(5, 5))
];

pub const TILE_SIZE: Size2D = Size2D::new(25, 11);

const TILE_RARITY_SYMBOL: &'static str = "⬤";

const TILE_SYMBOL_OFFSET: Point2D = Point2D::new((TILE_SIZE.x/2) as i16, 1);

lazy_static! {

    static ref DEFAULT_ROLL_NUMBER_CELL: Cell = {
        let mut cell = Cell::default();
        cell
            .set_char('@')
            .set_bg(Color::White)
            .set_fg(Color::White);
        cell
    };

    static ref BEST_ROLL_NUMBER_CELL: Cell = {
        let mut cell: Cell = DEFAULT_ROLL_NUMBER_CELL.clone();
        cell
            .set_bg(Color::Red)
            .set_fg(Color::Red);
        cell
    };

    static ref TILE_BKG_CELL: Cell = {
        let mut cell = Cell::default();
        cell.set_char('#');
        cell
    };

    static ref DEFAULT_ROLL_RARITY_CELL: Cell = {
        let mut cell = Cell::default();
        cell
            .set_symbol("⬤")
            .set_fg(Color::White);
        cell
    };

    static ref BEST_ROLL_RARITY_CELL: Cell = {
        let mut cell = Cell::default();
        cell
            .set_symbol("⬤")
            .set_fg(Color::Red);
        cell
    };

    static ref TILE_ROLL_RARITY_BITSHAPES: Vec<BitShape128> = {
        let mut shapes: Vec<BitShape128> = Vec::with_capacity(13);
        for roll in 0..13 {
            let rarity = (6 as u16).saturating_sub((7 as u16).abs_diff(roll));
            let width = (2*rarity).saturating_sub(1).max(1);
            let mut bits: u128 = 0;
            for _ in 0..rarity {
                bits = bits << 2 | 1;
            }

            shapes.push(BitShape128::new(bits, Size2D::new(width, 1)));
        }

        shapes
    };

    static ref TILE_BKG_BITSHAPE: BitShape = {
        let width = TILE_SIZE.x;
        let height = TILE_SIZE.y;

        let mut buf: Vec<u128> = Vec::with_capacity(height as usize);
        let max_width_y = (height + (height + 1) % 2)/2;

        for y in 0..height {
            let padding = max_width_y.abs_diff(y);
            let fill_width = width.saturating_sub(2*padding);
            let fill_bits: u128 = if fill_width == 0 { 0 } else { (u128::MAX >> (128 - fill_width)) << padding };
            buf.push(fill_bits);
        }

        BitShape::new(buf, TILE_SIZE)
    };

    static ref TILE_EMPTY_BKG_SHAPE: Shape<'static> = Shape::new(
        &TILE_BKG_BITSHAPE, 
        " ",
        Style::default().bg(MAP_SAND_COLOR),
        DrawLayout::default()
            .center()
            .clone()
    );
}

#[derive(Debug)]
pub struct Tile {
    pub layout: DrawLayout,
    pub resource: enums::TileResource,
    bkg: Shape<'static>,
    rarity: Option<Shape128>, 
    digits: (Option<Shape128>, Option<Shape128>),
    mount: Mount,
    anim: TileAnimationData
}

#[derive(Debug, Default)]
struct TileAnimationData {
    bkg_alpha: f32,
    duration: f32,
    start: Option<Instant>,
    is_done: bool
}

impl Animatable for Tile {
    type Target = ();
    fn step(&mut self, _: &mut Self::Target, service: &mut ScreenAnimationService) {
        if let Some(start) = self.anim.start {
            let alpha = (start.elapsed().as_secs_f32()/self.anim.duration).min(1.0);
            if alpha == 1.0 {
                self.anim.is_done = true;
                service.sub();
            }

            self.anim.bkg_alpha = ease(alpha, EasingStyle::Cubic, EasingDirection::InOut);
        }
    }

    // Tile animations aren't meant to be cancellable
    fn cancel(&mut self, _: &mut ScreenAnimationService) {}
}

impl Tile {
    pub fn play_animation(&mut self, service: &mut ScreenAnimationService) {
        if let None = self.anim.start {
            self.anim.duration = 0.5;
            self.anim.start = Some(Instant::now());
            service.add();
        }
    }

    pub fn new(roll: u8, resource: enums::TileResource) -> Self {
        let bkg = Shape::new(
            &TILE_BKG_BITSHAPE, 
            " ",
            Style::default().bg(resource.get_color()),
            DrawLayout::default()
                .center()
                .clone()
        );

        let layout = DrawLayout::default()
            .set_size(UDim2::from_size2d(TILE_SIZE))
            .clone();

        if resource == enums::TileResource::OfDesert {
            Tile {
                layout,
                resource,
                bkg,
                rarity: None,
                digits: (None, None),
                mount: Mount::default(),
                anim: TileAnimationData::default()
            }
        } else {
            let is_best_roll = roll.abs_diff(7) == 1;

            let rarity = Shape128::new(
                &TILE_ROLL_RARITY_BITSHAPES[roll as usize], 
                TILE_RARITY_SYMBOL,
                Style::default().fg(if is_best_roll { Color::Red } else { Color::White }),
                DrawLayout::default()
                    .set_position(UDim2::new(0.5, 0, 1.0, -1))
                    .set_anchor(Scale2D::new(0.5, 0.5))
                    .clone()
            );
    
            let mut digits = (None, None);
    
            if roll < 10 {
                digits.0 = Some(Shape128::new(
                    &DIGITS[roll as usize],
                    " ",
                    Style::default().bg(if is_best_roll { Color::Red } else { Color::White }),
                    DrawLayout::default()
                        .center()
                        .clone()
                ));
            } else {
                digits = (
                    Some(Shape128::new(
                        &DIGITS[(roll % 10) as usize], // left digit (roll % 10)
                        " ",
                    Style::default().bg(if is_best_roll { Color::Red } else { Color::White }),
                        DrawLayout::default()
                            .set_position(UDim2::new(0.5, 1, 0.5, 0))
                            .set_anchor(Scale2D::new(0.0, 0.5))
                            .clone()
                    )),
                    Some(Shape128::new(
                        &DIGITS[(roll as usize / 10) % 10],  // right digit (roll / 10 mod 10)
                        " ",
                        Style::default().bg(if is_best_roll { Color::Red } else { Color::White }),
                        DrawLayout::default()
                            .set_position(UDim2::new(0.5, -1, 0.5, 0))
                            .set_anchor(Scale2D::new(1.0, 0.5))
                            .clone()
                    ))
                );
            }
    
            Tile { 
                layout,
                resource,
                bkg,
                digits,
                rarity: Some(rarity),
                mount: Mount::default(),
                anim: TileAnimationData::default()
            }
        }
    }

    fn draw_area(&self, area: &mut ScreenArea) {
        area.draw_child(&self.bkg);
        
        if let Some(rarity) = self.rarity.as_ref() {
            area.draw_child(rarity);
        }
        if let Some(digit) = &self.digits.0 {
            area.draw_child(digit);
        }
        if let Some(digit) = &self.digits.1 {
            area.draw_child(digit);
        }
        area.draw_unicode_line(
            self.resource.get_symbol(), 
            TILE_SYMBOL_OFFSET, 
            Style::default().fg(self.resource.get_color())
        );
    }
}

impl Layoutable for Tile {
    fn layout_ref(&self) -> &DrawLayout { &self.layout }
    fn layout_mut(&mut self) -> &mut DrawLayout { &mut self.layout }
}

impl Drawable for Tile {
    fn draw(&self, mut area: ScreenArea) {
        if self.anim.bkg_alpha >= 1.0 {
            self.draw_area(&mut area);
        } else {
            area.draw_child(&*TILE_EMPTY_BKG_SHAPE);
            if self.anim.bkg_alpha > 0.0 {
                let mut tile_buf = Buffer::empty(Rect::new(0, 0, TILE_SIZE.x, TILE_SIZE.y));
                let mut tile_area = ScreenArea::from_buffer(&mut tile_buf);
                self.draw_area(&mut tile_area);

                let h = 0.5*TILE_SIZE.x as f32;
                let k = 0.5*TILE_SIZE.y as f32;
                let a = 0.5*self.anim.bkg_alpha*TILE_SIZE.x.max(2*TILE_SIZE.y) as f32; // 2*y because font height is twice font width
                let b = 0.5*a; // font height in terminal is twice width so to account for this the ellipsis y axis must be half the x axis

                let circle = BitShape::paint(TILE_SIZE, |x, y| ((x as f32 + 0.5 - h)/a).powi(2) + ((y as f32 + 0.5 - k)/b).powi(2) <= 1.0);
                tile_area.bitmask(&circle);
                area.overlay(&mut tile_buf);
            }
        }
    }
}

// test
use super::placement::Placement;
use crate::render::mount::*;

impl Placement for Tile {
    fn get_placement_space(&self) -> Space { self.layout.space }
    fn set_placement_style(&mut self, _: Style) {}
}

impl MountableLayout for Tile {
    fn mount_ref(&self) -> &Mount { &self.mount }
    fn mount_mut(&mut self) -> &mut Mount { &mut self.mount }
    fn child_ref(&self, _: usize) -> Option<&dyn MountableLayout> { None }
    fn child_mut(&mut self, _: usize) -> Option<&mut dyn MountableLayout> { None }

    fn relayout(&mut self, relayout: &mut ScreenRelayout) {
        if self.anim.start.is_some() && !self.anim.is_done {
            self.step(&mut (), relayout.animation);
        }
        self.default_relayout(relayout);
    }
}