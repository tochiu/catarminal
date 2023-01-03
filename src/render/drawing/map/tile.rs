use super::{MAP_SAND_COLOR, parse};

use crate::render::prelude::*;
use crate::enums;

use tui::{
    style::{Color, Style},
    buffer::Buffer, 
    layout::Rect
};

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
const TILE_SYMBOL_OFFSET: Point2D = Point2D::new((TILE_SIZE.x/2 - 1) as i16, 1);
const TILE_BEST_FONT_COLOR: Color = Color::Red;
const TILE_FONT_COLOR: Color = Color::White;

const TILE_ELEMENT_FALL_DURATION_PER_PIXEL: f32 = 0.025;

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
    anim: TileAnimation // TODO: Option<Box<>> it
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
                        .set_anchor(Float2D::new(0.0, 0.5))
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
                        .set_anchor(Float2D::new(1.0, 0.5))
                        .clone()
                ))
            }
        };

        Tile {
            layout,
            resource,
            is_best,
            bkg,
            anim: TileAnimation::new(rarity),
            digit0,
            digit1,
            mount: Mount::default()
        }
    }

    pub fn play(&mut self, anim_service: &mut AnimationService) {
        self.anim.play(anim_service);
    }

    fn draw_common(&self, area: &mut DrawContext) {
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

    /* util */
    pub fn get_map_fall_parameters(&self, goal: Point2D, height: i16) -> (Point2D, f32) {
        let absolute_tile_space = self.layout.space.to_absolute_space(AbsoluteSpace {
            size: parse::MAP_BKG_SHAPE.shape.size,
            position: Point2D::new(0, 0)
        });
        let absolute_goal = absolute_tile_space.absolute_position_of(goal);
        let duration = (TILE_ELEMENT_FALL_DURATION_PER_PIXEL*(absolute_goal.y as f32 - height as f32)).abs();
        (Point2D::new(absolute_goal.x, height), duration)
    }
}

impl Layoutable for Tile {
    fn layout_ref(&self) -> &DrawLayout { &self.layout }
    fn layout_mut(&mut self) -> &mut DrawLayout { &mut self.layout }
}

impl Drawable for Tile {
    fn draw(&self, ctx: &mut DrawContext) {
        let animator = &self.anim.animator;

        if animator.bkg_alpha == 1.0 {
            self.draw_common(ctx);
        } else {
            ctx.draw_child(&*TILE_EMPTY_BKG_SHAPE);
            if animator.bkg_alpha > 0.0 {
                // TODO: move buffer into a refcell in the animation struct so this doesnt get instanced every anim frame
                let mut tile_buf = Buffer::empty(Rect::new(0, 0, TILE_SIZE.x, TILE_SIZE.y));
                let mut tile_area = DrawContext::from_buffer(&mut tile_buf);

                self.draw_common(&mut tile_area);

                tile_area.retain(Ellipse::scaled_circle_painter(TILE_SIZE, 0.5*TILE_SIZE.to_float2d(), animator.bkg_alpha));
                ctx.overlay(&mut tile_buf);
            }
        }

        if self.resource != enums::TileResource::OfDesert {
            for (i, &alpha) in animator.rarity_alphas.iter().take(animator.rarity as usize).enumerate() {
                let start = Point2D::new(0, TILE_SIZE.y as i16 - 2);
                let end = Point2D::new(
                    (TILE_SIZE.x as i16 - (2*animator.rarity as i16 - 1))/2 + 2*(animator.rarity as usize - 1 - i) as i16, 
                    TILE_SIZE.y as i16 - 2
                );
                
                let pos = start.lerp(end, ease(alpha, EasingStyle::Cubic, EasingDirection::InOut));

                let x = u16::try_from(pos.x);
                let y = u16::try_from(pos.y);

                if x.is_ok() && y.is_ok() && self.bkg.bitshape.is_filled_at(x.unwrap(), y.unwrap()) {
                    if let Some(cell) = ctx.cell_at_mut(pos) {
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

    fn relayout(&mut self, ctx: &mut LayoutContext) {
        if self.anim.state.playback == PlaybackState::Playing {
            self.anim.update(&mut ());
        }
        ctx.relayout_children_of(self);
    }
}

#[derive(Debug, Default)]
struct TileAnimator {
    bkg_alpha: f32,
    rarity: u8,
    rarity_alphas: [f32; 6],
}

impl TileAnimator {
    const BKG_DURATION: f32 = 0.5;
    const ROLL_RARITY_DOT_DURATION: f32 = 1.0;
    const ROLL_RARITY_DOT_DELAY: f32 = 0.3;
    const ROLL_RARITY_START_TIME: f32 = 0.0;
}

type TileAnimation = Animation<TileAnimator>;

impl TileAnimation {
    fn new(rarity: u8) -> Self {
        let rarity_duration = if rarity > 0
            { TileAnimator::ROLL_RARITY_DOT_DURATION + TileAnimator::ROLL_RARITY_DOT_DELAY*(rarity.saturating_sub(1) as f32) } else
            { 0.0 };
        Animation::with_duration(
            f32::max(TileAnimator::BKG_DURATION, TileAnimator::ROLL_RARITY_START_TIME + rarity_duration), 
            TileAnimator { rarity, ..Default::default() }
        )
    }
}

impl Animator for TileAnimator {
    type Target = ();
    fn update(&mut self, state: &AnimationState, _: &mut Self::Target) {
        let elapsed = state.get_elapsed();
        self.bkg_alpha = ease((elapsed/Self::BKG_DURATION).min(1.0), EasingStyle::Cubic, EasingDirection::InOut);
        for (i, alpha_rarity) in self.rarity_alphas.iter_mut().take(self.rarity as usize).enumerate() {
            let alpha = (
                (elapsed
                    - TileAnimator::ROLL_RARITY_START_TIME 
                    - TileAnimator::ROLL_RARITY_DOT_DELAY*i as f32
                )/TileAnimator::ROLL_RARITY_DOT_DURATION
            ).clamp(0.0, 1.0);
            *alpha_rarity = ease(alpha, EasingStyle::Cubic, EasingDirection::InOut);
        }
    }
}

/* this is meant to be instanced by the map */
pub type TileDigitsAnimation = Animation<TileDigitsAnimator>;

#[derive(Debug)]
pub struct TileDigitsAnimator {
    pub digit0: Option<DigitAnimator>,
    pub digit1: Option<DigitAnimator>
}

impl TileDigitsAnimator {
    const DIGIT_DELAY: f32 = 0.3;
}

impl TileDigitsAnimation {
    pub fn new(tile: &Tile) -> Self {
        let digit0 = tile.digit0.as_ref().map(|digit| DigitAnimator::new(tile, digit, if tile.digit1.is_some() { TileDigitsAnimator::DIGIT_DELAY } else { 0.0 }));
        let digit1 = tile.digit1.as_ref().map(|digit| DigitAnimator::new(tile, digit, 0.0));

        let mut duration: f32 = 0.0;
        if let Some(digit) = digit0.as_ref() {
            duration = duration.max(digit.delay + digit.duration);
        }
        if let Some(digit) = digit1.as_ref() {
            duration = duration.max(digit.delay + digit.duration);
        }

        Animation::with_duration(duration, TileDigitsAnimator { digit0, digit1 })
    }
}

impl Animator for TileDigitsAnimator {
    type Target = Tile;

    fn update(&mut self, state: &AnimationState, target: &mut Self::Target) {
        let elapsed = state.get_elapsed();
        if let Some(anim) = self.digit0.as_mut() {
            anim.update(elapsed);
            if anim.done {
                self.digit0 = None;
                target.digit0.as_mut().unwrap().layout.set_visible(true);
            } else {
                target.digit0.as_mut().unwrap().layout.set_visible(false);
            }
        }

        if let Some(anim) = self.digit1.as_mut() {
            anim.update(elapsed);
            if anim.done {
                self.digit1 = None;
                target.digit1.as_mut().unwrap().layout.set_visible(true);
            } else {
                target.digit1.as_mut().unwrap().layout.set_visible(false);
            }
        }
    }
}

#[derive(Debug)]
pub struct DigitAnimator {
    space0: Space,
    space1: Space,
    duration: f32,
    delay: f32,
    pub digit: Shape128,
    done: bool,
    alpha: f32
}

impl DigitAnimator {
    fn new(tile: &Tile, digit: &Shape128, delay: f32) -> DigitAnimator {
        let absolute_tile_space = tile.layout.space.to_absolute_space(AbsoluteSpace {
            size: parse::MAP_BKG_SHAPE.shape.size,
            position: Point2D::new(0, 0)
        });
        let absolute_digit_space = digit.layout.space.to_absolute_space(absolute_tile_space);
        let (start, duration) = tile.get_map_fall_parameters(
            absolute_tile_space.relative_position_of(absolute_digit_space.position), 
            -(absolute_digit_space.size.y as i16)
        );
        
        DigitAnimator {
            space0: Space::new(
                UDim2::from_size2d(absolute_digit_space.size),
                UDim2::from_point2d(start),
                Float2D::default()
            ),
            space1: Space::new(
                UDim2::from_size2d(absolute_digit_space.size),
                UDim2::from_point2d(absolute_digit_space.position), 
                Float2D::default()
            ),
            duration,
            delay,
            digit: digit.clone(),
            done: false,
            alpha: 0.0
        }
    }

    pub fn update(&mut self, elapsed: f32) {
        if self.done {
            return
        }

        // check if elapsed is greater first to avoid precision bugs from calculating alpha
        let alpha = if elapsed >= self.delay + self.duration { 1.0 } else { ((elapsed - self.delay)/self.duration).max(0.0) };
        self.alpha = alpha;
        if alpha == 1.0 {
            self.done = true;
        } else {
            self.digit.layout.set_space(self.space0.lerp(self.space1, ease(alpha, EasingStyle::Cubic, EasingDirection::Out)));
        }
    }
}