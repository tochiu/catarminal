use super::parse::*;
use crate::render::prelude::*;

use crate::enums;

use tui::{style::{Color, Style}, buffer::Buffer, layout::Rect};

use std::cell::RefCell;

const PORT_RESOURCE_SYMBOL_OFFSET: Point2D = Point2D::new(-1, 0);
const PORT_RATIO_OFFSET: Point2D = Point2D::new(-1, 1);
const PORT_COLOR: Color = Color::Rgb(221, 149, 47);

lazy_static! {
    static ref PORT_BITSHAPES: Vec<(AbsoluteSpace, BitShape)> = {
        const PORT_ANCHOR_OFFSETS: [Point2D; 4] = [
            Point2D::new(2, 0),
            Point2D::new(-2, 0),
            Point2D::new(0, 1),
            Point2D::new(0, -1),
        ];
        MAP_GRAPH
            .port_points
            .iter()
            .enumerate()
            .map(|(port, &port_point)| {
                let points = MAP_GRAPH.port_plots[port]
                    .iter()
                    .map(|&plot| MAP_GRAPH.plot_points[plot])
                    .chain(
                        PORT_ANCHOR_OFFSETS
                            .iter()
                            .map(|&offset| port_point + offset),
                    )
                    .collect::<Vec<_>>();
                let absolute_port_space = AbsoluteSpace::from_point_cloud(&points);

                let bitshape = BitShape::paint(absolute_port_space.size, |x, y| {
                    let x = (x + absolute_port_space.position.x as u16) as usize;
                    let y = (y + absolute_port_space.position.y as u16) as usize;
                    if let Some("*" | "X" | "?") = MAP_BKG_DRAW_STRING
                        .lines
                        .get(y)
                        .and_then(|&line| line.get(x..=x))
                    {
                        true
                    } else {
                        false
                    }
                });

                (absolute_port_space, bitshape)
            })
            .collect()
    };
}

#[derive(Debug)]
pub struct Port {
    layout: DrawLayout,
    resource: enums::PortResource,
    ratio: String,
    shape: Shape<'static>,
    mark: Point2D,
    anim: Option<Box<PortAnimation>>,
    mount: Mount
}

#[derive(Debug)]
struct PortAnimation(RefCell<Buffer>, Animation<()>);

impl PortAnimation {
    const DURATION: f32 = 2.0;
}

impl Port {
    pub fn new(port: usize, resource: enums::PortResource) -> Self {
        let (num_give, num_take) = resource.get_ratio();
        Port {
            resource,
            mark: PORT_BITSHAPES[port].0.relative_position_of(MAP_GRAPH.port_points[port]),
            anim: None,
            mount: Mount::default(),
            ratio: [
                char::from_digit(num_give, 10).unwrap(),
                ':',
                char::from_digit(num_take, 10).unwrap(),
            ].iter().collect(),
            shape: Shape::new(
                &PORT_BITSHAPES[port].1,
                " ",
                Style::default().bg(PORT_COLOR),
                DrawLayout::default(),
            ),
            layout: DrawLayout::default()
                .set_size(UDim2::from_size2d(PORT_BITSHAPES[port].0.size))
                .set_position(UDim2::from_point2d(PORT_BITSHAPES[port].0.position))
                .set_visible(false)
                .clone(),
        }
    }

    pub fn draw(&self, ctx: &mut DrawContext) {
        ctx.draw_child(&self.shape);
        ctx.draw_unicode_line(
            &(self.resource.get_symbol().to_owned() + "?"),
            self.mark + PORT_RESOURCE_SYMBOL_OFFSET,
            Style::default().fg(Color::White),
        );
        ctx.draw_string_line(
            &self.ratio,
            self.mark + PORT_RATIO_OFFSET,
            Style::default().fg(Color::White),
        );
    }

    pub fn animate(&mut self, anim_service: &mut AnimationService) {
        self.layout.set_visible(true);
        let mut anim = Box::new(PortAnimation(
            RefCell::new(Buffer::empty(Rect::new(0, 0, self.shape.bitshape.size.x, self.shape.bitshape.size.y))),
            Animation::with_duration(PortAnimation::DURATION, ())
        ));
        anim.1.play(anim_service);
        self.anim = Some(anim);
    }
}

impl Layoutable for Port {
    fn layout_ref(&self) -> &DrawLayout { &self.layout }
    fn layout_mut(&mut self) -> &mut DrawLayout { &mut self.layout }
}

impl Drawable for Port {
    fn draw(&self, ctx: &mut DrawContext) {
        if let Some(anim) = self.anim.as_deref() {
            let alpha = anim.1.state.get_alpha();
            if alpha == 1.0 {
                self.draw(ctx);
            } else if alpha > 0.0 {
                let mut workspace = anim.0.borrow_mut();
                let mut workspace_ctx = DrawContext::from_buffer(&mut workspace);

                self.draw(&mut workspace_ctx);

                let workspace_size = Size2D::new(workspace_ctx.buf.area.width, workspace_ctx.buf.area.height);
                workspace_ctx.retain(Ellipse::scaled_circle_painter(workspace_size, self.mark.to_float2d(), 2.0*alpha));
                ctx.overlay(workspace_ctx.buf);
            }
        } else {
            self.draw(ctx)
        }
    }
}

impl MountableLayout for Port {
    fn mount_ref(&self) -> &Mount { &self.mount }
    fn mount_mut(&mut self) -> &mut Mount { &mut self.mount }
    fn child_ref(&self, _: usize) -> Option<&dyn MountableLayout> { None }
    fn child_mut(&mut self, _: usize) -> Option<&mut dyn MountableLayout> { None }

    fn relayout(&mut self, ctx: &mut LayoutContext) {
        if let Some(anim) = self.anim.as_deref_mut() {
            anim.1.update(&mut ());
            if anim.1.state.playback != PlaybackState::Playing {
                self.anim = None;
            }
        }

        ctx.relayout_children_of(self)
    }
}