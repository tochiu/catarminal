use std::{iter, path::Path};

use super::{draw::*, prelude::{Animation, Animator, EasingDirection, EasingStyle}, space::*};

use image::{self, GenericImageView, Rgba};
use tui::style::Color;

#[derive(Debug, Clone)]
pub struct Sprite {
    pub layout: DrawLayout,
    pub colors: Vec<Option<Color>>,
    pub size: Size2D,
}

impl TryFrom<&Path> for Sprite {
    type Error = String;
    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let img = image::open(path).map_err(|e| e.to_string())?;
        let size = Size2D::new(img.width() as u16, img.height().next_multiple_of(2) as u16);
        let mut layout = DrawLayout::default();
        layout.set_size(UDim2::from_size2d(Size2D::new(size.x, size.y / 2)));

        let colors = img
            .pixels()
            .map(|(_, _, Rgba([r, g, b, a]))| {
                if a == 0 {
                    None
                } else {
                    Some(Color::Rgb(r, g, b))
                }
            })
            .chain(iter::once(None).cycle().take(if img.height() % 2 == 1 {
                size.x as usize
            } else {
                0
            }))
            .collect();

        Ok(Sprite {
            layout,
            colors,
            size,
        })
    }
}

impl Layoutable for Sprite {
    fn layout_ref(&self) -> &DrawLayout {
        &self.layout
    }
    fn layout_mut(&mut self) -> &mut DrawLayout {
        &mut self.layout
    }
}

impl Drawable for Sprite {
    fn draw(&self, ctx: &mut DrawContext) {
        for point in ctx.absolute_draw_space.intersection(AbsoluteSpace {
            size: Size2D::new(self.size.x, self.size.y / 2),
            position: ctx.absolute_draw_space.position,
        }) {
            let bit_point = ctx.absolute_layout_space.relative_position_of(point);

            let top =
                self.colors[((bit_point.y as u16 * 2 + 0) * self.size.x + bit_point.x as u16) as usize];
            let bot =
                self.colors[((bit_point.y as u16 * 2 + 1) * self.size.x + bit_point.x as u16) as usize];

            if top.is_some() || bot.is_some() {
                let index = ctx.buf.index_of(point.x as u16, point.y as u16);
                let pixel = &mut ctx.buf.content[index];

                if let Some(top_color) = top {
                    if let Some(bot_color) = bot {
                        if top_color == bot_color {
                            pixel.set_symbol(" ").set_bg(bot_color);
                        } else {
                            pixel.set_symbol("▀").set_bg(bot_color).set_fg(top_color);
                        }
                    }
                    else {
                        pixel.set_symbol("▀").set_fg(top_color);
                    }
                } else if let Some(bot_color) = bot {
                    pixel.set_symbol("▄").set_fg(bot_color);
                }
            }
        }
    }
}

// pub struct Flipbook {
//     pub frames: Vec<Sprite>,
//     pub current_frame: usize,
// }

// pub type FlipbookAnimation<'a> = Animation<FlipbookAnimator<'a>>;

// impl<'a> FlipbookAnimation<'a> {
//     pub fn new(duration: f32, style: EasingStyle, direction: EasingDirection) -> Self {
//         Animation::with_duration(duration, FlipbookAnimator { frames, style, direction })
//     }
// }

// #[derive(Debug)]
// struct SpriteAnimator<'a> {
//     pub frames: &'a [Sprite],
//     pub style: EasingStyle,
//     pub direction: EasingDirection,
// }

// impl<'a> Animator for SpriteAnimator<'_> {
//     type Target = Sprite;

//     fn play(&mut self) {
//         self.current_frame = 0;
//     }
// }

// impl Animator for SpaceAnimator {
//     type Target = Space;

//     /* update animation state and target space accordingly */
//     fn update(&mut self, state: &AnimationState, target: &mut Self::Target) {
//         /* update target acoordingly (this still runs when alpha == 1 to put target at goal space) */
//         *target = self.space0.lerp(self.space1, ease(state.get_alpha(), self.style, self.direction));
//     }
// }