use crate::render::{draw::*, space::*};

use tui::style::Style;

pub const COUNTER_SIZE: Size2D = Size2D::new(4, 3);

const CARD_COUNTER_TEXT_OFFSET: Point2D = Point2D::new(2, 0);
const DEFAULT_COUNTER_TEXT_OFFSET: Point2D = Point2D::new(1, 2);
const SYMBOL_OFFSET: Point2D = Point2D::new(1, 1);

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum SymbolCounterKind {
    Default,
    Card
}

#[derive(Copy, Clone, Debug)]
pub struct SymbolCounterStyle {
    pub symbol: Style,
    pub text: Style,
    pub bkg: Style,
}

#[derive(Debug)]
pub struct SymbolCounter {
    symbol: String,
    kind: SymbolCounterKind,
    style: SymbolCounterStyle,

    pub layout: DrawLayout
}

impl SymbolCounter {
    pub fn new(symbol: String, kind: SymbolCounterKind, style: SymbolCounterStyle, mut layout: DrawLayout) -> Self {
        layout.set_size(UDim2::from_size2d(COUNTER_SIZE));
        SymbolCounter {
            symbol,
            kind,
            style,
            layout
        }
    }
}

impl Layoutable for SymbolCounter {
    fn layout_ref(&self) -> &DrawLayout { &self.layout }
    fn layout_mut(&mut self) -> &mut DrawLayout { &mut self.layout }
}

impl StatefulDrawable for SymbolCounter {
    type State = u8;
    fn stateful_draw(&self, ctx: &mut DrawContext, count: &u8) {
        if self.kind == SymbolCounterKind::Card {
            ctx.transform_cells(|cell| {
                cell.set_style(self.style.bkg); 
            });
        }
        
        ctx.draw_unicode_line(&self.symbol, SYMBOL_OFFSET, self.style.symbol);
        ctx.draw_string_line(
            &format!("{:0width$}", (*count).min(99), width = 2 as usize), 
            if self.kind == SymbolCounterKind::Default
                { DEFAULT_COUNTER_TEXT_OFFSET } else 
                { CARD_COUNTER_TEXT_OFFSET }, 
            self.style.text
        );
    }
}