use super::{super::{
    shape::*,
    super::{
        draw::*,
        space::*,
        screen::*,
        anim::*,
        mount::*
    }
}, MAP_SAND_COLOR};

use crate::enums;

use tui::{
    style::{Color, Style},
    buffer::Buffer, 
    layout::Rect
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
const TILE_BEST_FONT_COLOR: Color = Color::Red;
const TILE_FONT_COLOR: Color = Color::White;

lazy_static! {

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
        &TILE_BKG_BITSHAPE, " ",
        Style::default().bg(MAP_SAND_COLOR),
        DrawLayout::default().center().clone()
    );
}

#[derive(Debug)]
pub struct Tile {
    pub layout: DrawLayout,
    pub resource: enums::TileResource,
    is_best: bool,
    bkg: Shape<'static>,
    digit0: Option<Shape128>,
    digit1: Option<Shape128>,
    mount: Mount,
    anim: TileAnimation
}

impl Tile {
    pub fn new(roll: u8, resource: enums::TileResource) -> Self {
        let rarity = 6 - roll.abs_diff(7);
        let is_best = rarity >= 5;

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
        
        let digit_style = Style::default().bg(if is_best { TILE_BEST_FONT_COLOR } else { TILE_FONT_COLOR });
        let digit0 = {
            if resource == enums::TileResource::OfDesert {
                None
            } else if roll < 10 {
                Some(Shape128::new(
                    &DIGITS[roll as usize], " ", digit_style,
                    DrawLayout::default().center().clone()
                ))
            } else {
                Some(Shape128::new(
                    &DIGITS[(roll % 10) as usize], " ", digit_style,
                    DrawLayout::default()
                        .set_position(UDim2::new(0.5, 1, 0.5, 0))
                        .set_anchor(Scale2D::new(0.0, 0.5))
                        .clone()
                ))
            }
        };
        let digit1 = {
            if resource == enums::TileResource::OfDesert || roll < 10 {
                None
            } else {
                Some(Shape128::new(
                    &DIGITS[(roll as usize / 10) % 10],  " ", digit_style,
                    DrawLayout::default()
                        .set_position(UDim2::new(0.5, -1, 0.5, 0))
                        .set_anchor(Scale2D::new(1.0, 0.5))
                        .clone()
                ))
            }
        };

        Tile {
            layout,
            resource,
            is_best,
            bkg,
            anim: TileAnimation::new(
                rarity, 
                digit0.as_ref().map(|digit0| digit0.layout.space), 
                digit1.as_ref().map(|digit1| digit1.layout.space)
            ),
            digit0,
            digit1,
            mount: Mount::default()
        }
    }

    pub fn play(&mut self, anim_service: &mut ScreenAnimationService) {
        self.anim.play(anim_service);
    }

    fn draw_area(&self, area: &mut ScreenArea) {
        area.draw_child(&self.bkg);
        area.draw_unicode_line(
            self.resource.get_symbol(), 
            TILE_SYMBOL_OFFSET, 
            Style::default().fg(self.resource.get_color())
        );

        if let Some(digit) = &self.digit0 {
            area.draw_child(digit);
        }
        if let Some(digit) = &self.digit1 {
            area.draw_child(digit);
        }
    }
}

impl Layoutable for Tile {
    fn layout_ref(&self) -> &DrawLayout { &self.layout }
    fn layout_mut(&mut self) -> &mut DrawLayout { &mut self.layout }
}

impl Drawable for Tile {
    fn draw(&self, mut area: ScreenArea) {
        if self.anim.bkg_alpha == 1.0 {
            self.draw_area(&mut area);
        } else {
            area.draw_child(&*TILE_EMPTY_BKG_SHAPE);
            if self.anim.bkg_alpha > 0.0 {
                let mut tile_buf = Buffer::empty(Rect::new(0, 0, TILE_SIZE.x, TILE_SIZE.y));
                let mut tile_area = ScreenArea::from_buffer(&mut tile_buf);
                self.draw_area(&mut tile_area);

                let a = 0.5*self.anim.bkg_alpha*TILE_SIZE.x.max(2*TILE_SIZE.y) as f32; // 2*y because font height is twice font width
                let b = 0.5*a; // font height in terminal is twice width so to account for this the ellipsis y axis must be half the x axis

                tile_area.bitmask(&Ellipse::bits(TILE_SIZE, a, b));
                area.overlay(&mut tile_buf);
            }
        }

        if self.resource != enums::TileResource::OfDesert {
            for (i, &alpha) in self.anim.rarity_alphas.iter().take(self.anim.rarity as usize).enumerate() {
                let start = Point2D::new(0, TILE_SIZE.y as i16 - 2);
                let end = Point2D::new(
                    (TILE_SIZE.x as i16 - (2*self.anim.rarity as i16 - 1))/2 + 2*(self.anim.rarity as usize - 1 - i) as i16, 
                    TILE_SIZE.y as i16 - 2
                );
                
                let pos = start.lerp(end, ease(alpha, EasingStyle::Cubic, EasingDirection::InOut));

                let x = u16::try_from(pos.x);
                let y = u16::try_from(pos.y);

                if x.is_ok() && y.is_ok() && self.bkg.shape.is_filled_at(x.unwrap(), y.unwrap()) {
                    if let Some(cell) = area.mut_cell_at(pos) {
                        cell.set_symbol(TILE_RARITY_SYMBOL);
                        cell.set_fg(if self.is_best { TILE_BEST_FONT_COLOR } else { TILE_FONT_COLOR });
                    }   
                }
            }
        }
    }
}

impl MountableLayout for Tile {
    fn mount_ref(&self) -> &Mount { &self.mount }
    fn mount_mut(&mut self) -> &mut Mount { &mut self.mount }
    fn child_ref(&self, _: usize) -> Option<&dyn MountableLayout> { None }
    fn child_mut(&mut self, _: usize) -> Option<&mut dyn MountableLayout> { None }

    fn relayout(&mut self, relayout: &mut ScreenRelayout) {
        if self.anim.playback == PlaybackState::Playing {
            self.anim.step(&mut (), relayout.animation);
        }
        if let Some(digit0) = self.digit0.as_mut() {
            digit0.layout.set_space(self.anim.digit0_spaces.unwrap().0.lerp(self.anim.digit0_spaces.unwrap().1, self.anim.digit0_alpha));
        }
        if let Some(digit1) = self.digit1.as_mut() {
            digit1.layout.set_space(self.anim.digit1_spaces.unwrap().0.lerp(self.anim.digit1_spaces.unwrap().1, self.anim.digit1_alpha));
        }
        self.default_relayout(relayout);
    }
}

// TODO: implementation could be cleaner but this requires a redesign of the animation service and anim.rs

#[derive(Debug, Default)]
struct TileAnimation {
    bkg_alpha: f32,

    rarity: u8,
    rarity_alphas: [f32; 6],

    digit0_spaces: Option<(Space, Space)>,
    digit0_alpha: f32,

    digit1_spaces: Option<(Space, Space)>,
    digit1_alpha: f32,
    
    duration: f32,
    start: Option<Instant>,
    playback: PlaybackState
}

impl TileAnimation {
    const BKG_DURATION: f32 = 0.5;
    const DIGIT_DURATION: f32 = 0.5;
    const DIGIT_DELAY: f32 = 0.3;
    const ROLL_RARITY_DOT_DURATION: f32 = 1.0;
    const ROLL_RARITY_DOT_DELAY: f32 = 0.3;

    fn new(rarity: u8, digit0_space: Option<Space>, digit1_space: Option<Space>) -> Self {
        let rarity_duration = if rarity > 0
            { Self::ROLL_RARITY_DOT_DURATION + Self::ROLL_RARITY_DOT_DELAY*(rarity.saturating_sub(1) as f32) } else
            { 0.0 };
        let digits_duration = if digit0_space.is_some() 
            { Self::DIGIT_DURATION + if digit1_space.is_some() { Self::DIGIT_DELAY } else { 0.0 }} else 
            { 0.0 };

        let transform = |space: Space| (Space::new(space.size, UDim2 { x: space.position.x, y: UDim::new(0.0, -1) }, Scale2D::new(space.anchor.x, 1.0)), space);

        TileAnimation {
            rarity,
            digit0_spaces: digit0_space.map(transform),
            digit0_alpha: 0.0,
            digit1_spaces: digit1_space.map(transform),
            digit1_alpha: 0.0,
            duration: Self::BKG_DURATION + f32::max(rarity_duration, digits_duration),
            ..Default::default()
        }
    }

    // assumed to be called once per tile instance
    fn play(&mut self, anim_service: &mut ScreenAnimationService) {
        self.playback = PlaybackState::Playing;
        self.start = Some(Instant::now());
        anim_service.add();
    }
}

impl Animatable for TileAnimation {
    type Target = ();

    fn step(&mut self, _: &mut Self::Target, service: &mut ScreenAnimationService) {
        let elapsed = self.start.unwrap().elapsed().as_secs_f32().min(self.duration);
        if elapsed == self.duration {
            self.playback = PlaybackState::Stopped;
            service.sub();
        }

        self.bkg_alpha = ease((elapsed/Self::BKG_DURATION).min(1.0), EasingStyle::Cubic, EasingDirection::InOut);
        for (i, alpha_rarity) in self.rarity_alphas.iter_mut().take(self.rarity as usize).enumerate() {
            let alpha = (
                (elapsed
                    - Self::BKG_DURATION 
                    - Self::ROLL_RARITY_DOT_DELAY*i as f32
                )/Self::ROLL_RARITY_DOT_DURATION
            ).min(1.0).max(0.0);
            *alpha_rarity = ease(alpha, EasingStyle::Cubic, EasingDirection::InOut);
        }

        if self.digit0_spaces.is_some() {
            if self.digit1_spaces.is_some() {
                if self.digit0_spaces.is_some() {
                    let alpha = ((elapsed - Self::BKG_DURATION - Self::DIGIT_DELAY)/Self::DIGIT_DURATION).min(1.0).max(0.0);
                    self.digit0_alpha = ease(alpha, EasingStyle::Cubic, EasingDirection::Out);
                }
        
                if self.digit1_spaces.is_some() {
                    let alpha = ((elapsed - Self::BKG_DURATION)/Self::DIGIT_DURATION).min(1.0).max(0.0);
                    self.digit1_alpha = ease(alpha, EasingStyle::Cubic, EasingDirection::Out);
                }
            } else {
                if self.digit0_spaces.is_some() {
                    let alpha = ((elapsed - Self::BKG_DURATION)/Self::DIGIT_DURATION).min(1.0).max(0.0);
                    self.digit0_alpha = ease(alpha, EasingStyle::Cubic, EasingDirection::Out);
                }
            }
        }
    }

    fn cancel(&mut self, service: &mut ScreenAnimationService) {
        self.playback = PlaybackState::Stopped;
        service.sub();
    }
}