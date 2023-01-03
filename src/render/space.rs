/*
 * space.rs
 * module of constructs that define spatial coordinates in absolute or relative terms 
 * and methods that transform or query said space
 * 
 * a common theme in this module is to panic on overflowing operations
 */

use std::cmp::{max, min};
use std::ops::{Add, Sub, Mul};
use tui::layout::Rect;

/* 
 * Linear interpolation: 
 * structs that implement this trait can be interpolated 
 * where an alpha ([0, 1] scalar) defines interpolation progress 
 */
pub trait Lerp {
    fn lerp(self, to: Self, alpha: f32) -> Self;
}

// 2D Point
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Default)]
pub struct Point2D {
    pub x: i16,
    pub y: i16
}

impl Point2D {
    pub const fn new(x: i16, y: i16) -> Self {
        Point2D { x, y }
    }

    pub const fn to_float2d(self) -> Float2D {
        Float2D::new(self.x as f32, self.y as f32)
    }
}

impl Add<Point2D> for Point2D {
    type Output = Point2D;
    fn add(self, rhs: Point2D) -> Point2D {
        Point2D { 
            // avoid overflow
            x: self.x.checked_add(rhs.x).unwrap(), 
            y: self.y.checked_add(rhs.y).unwrap()
        }
    }
}

impl Sub<Point2D> for Point2D {
    type Output = Point2D;
    fn sub(self, rhs: Point2D) -> Point2D {
        Point2D { 
            // avoid overflow
            x: self.x.checked_sub(rhs.x).unwrap(), 
            y: self.y.checked_sub(rhs.y).unwrap()
        }
    }
}

impl Lerp for Point2D {
    fn lerp(self, to: Self, alpha: f32) -> Self {
        Point2D { 
            x: self.x + ((to.x as f32 - self.x as f32)*alpha).round() as i16,
            y: self.y + ((to.y as f32 - self.y as f32)*alpha).round() as i16
        }
    }
}

// 2D Size
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Default)]
pub struct Size2D {
    pub x: u16,
    pub y: u16
}

impl Size2D {
    pub const fn new(x: u16, y: u16) -> Self {
        Size2D { x, y }
    }

    pub const fn to_float2d(self) -> Float2D {
        Float2D::new(self.x as f32, self.y as f32)
    }

    pub fn to_rect(self) -> Rect {
        Rect::new(0, 0, self.x, self.y)
    }

    pub fn area(self) -> u16 {
        self.x.checked_mul(self.y).unwrap()
    }
}

impl Add<Size2D> for Size2D {
    type Output = Size2D;
    fn add(self, rhs: Size2D) -> Size2D {
        Size2D { 
            x: self.x.checked_add(rhs.x).unwrap(), 
            y: self.y.checked_add(rhs.y).unwrap()
        }
    }
}

impl Sub<Size2D> for Size2D {
    type Output = Size2D;
    fn sub(self, rhs: Size2D) -> Size2D {
        Size2D { 
            // avoid overflow
            x: self.x.checked_sub(rhs.x).unwrap(), 
            y: self.y.checked_sub(rhs.y).unwrap()
        }
    }
}

impl Lerp for Size2D {
    fn lerp(self, to: Self, alpha: f32) -> Self {
        Size2D { 
            x: self.x + ((to.x as f32 - self.x as f32)*alpha).round() as u16,
            y: self.y + ((to.x as f32 - self.x as f32)*alpha).round() as u16
        }
    }
}

// 2D f32
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Float2D {
    pub x: f32,
    pub y: f32
}

impl Float2D {
    pub const CENTER: Float2D = Float2D::new(0.5, 0.5);
    pub const fn new(x: f32, y: f32) -> Self {
        Float2D { x, y }
    }
}

impl Lerp for Float2D {
    fn lerp(self, to: Self, alpha: f32) -> Self {
        Float2D { 
            x: self.x + (to.x - self.x)*alpha, 
            y: self.y + (to.y - self.y)*alpha
        }
    }
}

impl Mul<f32> for Float2D {
    type Output = Float2D;
    fn mul(self, rhs: f32) -> Self::Output {
        Float2D::new(rhs*self.x, rhs*self.y)
    }
}

impl Mul<Float2D> for f32 {
    type Output = Float2D;
    fn mul(self, rhs: Float2D) -> Self::Output {
        rhs*self
    }
}

/* Stands for "Universal Dimension"
 * this struct is a copy of Roblox's UDim datatype used in UI classes but this construct is universal in all UI layout technology
 * scale is a metric defined relative to some "parent" space (ex: 0.5 scale could mean use half of the canvas space in one dimension)
 * offset is an metric defined in absolute pixels
 * UDim::new(0.5, 5) could mean use half the canvas space in one dimension + 5 pixels
 */
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct UDim {
    pub scale: f32,
    pub offset: i16
}

impl UDim {
    pub const fn new(scale: f32, offset: i16) -> Self {
        UDim { scale, offset }
    }
}

impl Lerp for UDim {
    fn lerp(self, to: Self, alpha: f32) -> Self {
        UDim { 
            scale: self.scale + (to.scale - self.scale)*alpha, 
            offset: self.offset + ((to.offset as f32 - self.offset as f32)*alpha).round() as i16
        }
    }
}

impl Add<UDim> for UDim {
    type Output = UDim;
    fn add(self, rhs: UDim) -> Self::Output {
        UDim::new(self.scale + rhs.scale, self.offset + rhs.offset)
    }
}

impl Sub<UDim> for UDim {
    type Output = UDim;
    fn sub(self, rhs: UDim) -> Self::Output {
        UDim::new(self.scale - rhs.scale, self.offset - rhs.offset)
    }
}

/*
 * 2D variant of UDim
 * This is the main struct used to define draw / layout spaces with respect to a parent space
 */
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct UDim2 {
    pub x: UDim,
    pub y: UDim
}

impl UDim2 {

    pub const CENTER: UDim2 = UDim2::from_scale(0.5, 0.5);

    pub const fn new(x_scale: f32, x_offset: i16, y_scale: f32, y_offset: i16) -> Self {
        UDim2 {
            x: UDim::new(x_scale, x_offset),
            y: UDim::new(y_scale, y_offset)
        }
    }

    pub const fn from_scale(x_scale: f32, y_scale: f32) -> Self {
        UDim2::new(x_scale, 0, y_scale, 0)
    }

    pub const fn from_offset(x_offset: i16, y_offset: i16) -> Self {
        UDim2::new(0.0, x_offset, 0.0, y_offset)
    }

    pub const fn from_point2d(point: Point2D) -> Self {
        UDim2::from_offset(point.x, point.y)
    }

    pub fn from_size2d(size: Size2D) -> Self {
        UDim2::from_offset(
            i16::try_from(size.x).unwrap(), 
            i16::try_from(size.y).unwrap()
        )
    }
}

impl Add<UDim2> for UDim2 {
    type Output = UDim2;
    fn add(self, rhs: UDim2) -> Self::Output {
        UDim2 { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}

impl Sub<UDim2> for UDim2 {
    type Output = UDim2;
    fn sub(self, rhs: UDim2) -> Self::Output {
        UDim2 { x: self.x - rhs.x, y: self.y - rhs.y }
    }
}

impl Lerp for UDim2 {
    fn lerp(self, to: Self, alpha: f32) -> Self {
        UDim2 { 
            x: self.x.lerp(to.x, alpha), 
            y: self.y.lerp(to.y, alpha) 
        }
    }
}

/*
 * AbsoluteSpace
 * Defines in exact pixel terms some canvas space
 * where size is the canvas size and position is the location of the top-left pixel of the canvas
 * Analgous to tui's Rect struct
 */
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Default)]
pub struct AbsoluteSpace {
    pub size: Size2D,
    pub position: Point2D
}

impl AbsoluteSpace {

    pub const fn new(x: i16, y: i16, width: u16, height: u16) -> Self {
        AbsoluteSpace {
            size: Size2D {
                x: width,
                y: height
            },
            position: Point2D { x, y }
        }
    }

    pub fn from_rect(rect: Rect) -> Self {
        AbsoluteSpace::new(
            i16::try_from(rect.x).unwrap(), 
            i16::try_from(rect.y).unwrap(), 
            rect.width, 
            rect.height
        )
    }

    pub fn from_point_cloud(points: &[Point2D]) -> Self {
        let mut point_iter = points.iter();
        let mut top_left = *point_iter.next().unwrap();
        let mut bottom_right = top_left;

        for point in point_iter {
            top_left.x = top_left.x.min(point.x);
            top_left.y = top_left.y.min(point.y);
            bottom_right.x = bottom_right.x.max(point.x);
            bottom_right.y = bottom_right.y.max(point.y);
        }

        AbsoluteSpace {
            position: top_left,
            size: Size2D::new(
                (bottom_right.x - top_left.x + 1) as u16, 
                (bottom_right.y - top_left.y + 1) as u16
            )
        }
    }

    pub fn area(self) -> u16 {
        self.size.x.checked_mul(self.size.y).unwrap()
    }

    pub fn left(self) -> i16 {
        self.position.x
    }

    pub fn right(self) -> i16 {
        i16::try_from(self.left() as i32 + self.size.x as i32).unwrap()
    }

    pub fn top(self) -> i16 {
        self.position.y
    }

    pub fn bottom(self) -> i16 {
        i16::try_from(self.top() as i32 + self.size.y as i32).unwrap()
    }

    pub fn intersection(self, other: AbsoluteSpace) -> AbsoluteSpace {
        let x1 = max(self.left(), other.left());
        let y1 = max(self.top(), other.top());
        let x2 = min(self.right(), other.right());
        let y2 = min(self.bottom(), other.bottom());
        
        AbsoluteSpace::new(
            x1, y1, 
            u16::try_from(x2 as i32 - x1 as i32).unwrap(), 
            u16::try_from(y2 as i32 - y1 as i32).unwrap()
        )
    }

    pub fn try_intersection(self, other: AbsoluteSpace) -> Option<AbsoluteSpace> {
        if self.intersects(other) {
            Some(self.intersection(other))
        } else {
            None
        }
    }

    pub fn intersects(self, other: AbsoluteSpace) -> bool {
        self.left() < other.right()
            && self.right() > other.left()
            && self.top() < other.bottom()
            && self.bottom() > other.top()
    }

    pub fn absolute_position_of(self, relative_position: Point2D) -> Point2D {
        Point2D {
            x: relative_position.x.checked_add(self.position.x).unwrap(),
            y: relative_position.y.checked_add(self.position.y).unwrap()
        }
    }

    pub fn relative_position_of(self, absolute_position: Point2D) -> Point2D {
        Point2D {
            x: absolute_position.x.checked_sub(self.position.x).unwrap(),
            y: absolute_position.y.checked_sub(self.position.y).unwrap()
        }
    }

    pub fn absolute_space_of(self, absolute_space: AbsoluteSpace) -> AbsoluteSpace {
        AbsoluteSpace {
            size: absolute_space.size,
            position: self.absolute_position_of(absolute_space.position)
        }
    }

    pub fn is_interior_point(self, point: Point2D) -> bool {
        point.x >= self.left() && point.x < self.right() && point.y >= self.top() && point.y < self.bottom()
    }

    pub fn to_rect(self) -> Rect {
        Rect::new(
            u16::try_from(self.position.x).unwrap(), 
            u16::try_from(self.position.y).unwrap(), 
            self.size.x, 
            self.size.y
        )
    }
}

impl Lerp for AbsoluteSpace {
    fn lerp(self, to: Self, alpha: f32) -> Self {
        AbsoluteSpace { 
            size: self.size.lerp(to.size, alpha), 
            position: self.position.lerp(to.position, alpha) 
        }
    }
}

/* implement IntoIterator for AbsoluteSpace to interate through all the (x, y) points within the region */
impl IntoIterator for AbsoluteSpace {
    type Item = Point2D;
    type IntoIter = AbsoluteSpaceIterator;

    fn into_iter(self) -> Self::IntoIter {
        AbsoluteSpaceIterator {
            space: self,
            index: 0,
            area: self.area()
        }
    }
}

/* This is the iterator that returns the next point inside the space on each call */
pub struct AbsoluteSpaceIterator {
    space: AbsoluteSpace,
    index: u16,
    area: u16
}

impl Iterator for AbsoluteSpaceIterator {
    type Item = Point2D;
    fn next(&mut self) -> Option<Point2D> {
        let index = self.index;
        if index < self.area {
            self.index += 1;
            Some(
                Point2D::new(
                    i16::try_from(self.space.position.x as i32 + (index % self.space.size.x) as i32).unwrap(),
                    i16::try_from(self.space.position.y as i32 + (index / self.space.size.x) as i32).unwrap()
                )
            )
        } else {
            None
        }
    }
}

/*
 * Space
 * Premiere struct that fully defines a layout relative to a parent layout
 * Size: size of the layout in relative terms
 * Position: position of the layout in relative terms
 * Anchor: a 2D scalar that defines what part of the layout will lie at the defined position
 *      ex: position (0.5 xscale, 0.5 yscale) anchor (0, 0) positions the top left point of the layout in the middle of the parent layout
 *          but anchor (0.5, 0.5) instead makes the middle point of the layout in the middle of the parent layout (thereby centering it)
 */
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Space {
    pub size: UDim2,
    pub position: UDim2,
    pub anchor: Float2D
}

impl Space {

    pub const FULL: Space = Space::sized(UDim2::from_scale(1.0, 1.0));

    pub const fn new(size: UDim2, position: UDim2, anchor: Float2D) -> Self {
        Space { size, position, anchor }
    }

    pub const fn sized(size: UDim2) -> Self {
        Space::new(size, UDim2::new(0.0, 0, 0.0, 0), Float2D::new(0.0, 0.0))
    }

    /* for a given parent AbsoluteSpace, compute the AbsoluteSpace of this Space */
    pub fn to_absolute_space(self, parent: AbsoluteSpace) -> AbsoluteSpace {
        /* absolute size values are just the scale*parent_size + offset */
        let sizex = (parent.size.x as f32)*self.size.x.scale + self.size.x.offset as f32;
        let sizey = (parent.size.y as f32)*self.size.y.scale + self.size.y.offset as f32;
        let sizex_abs = sizex.abs();
        let sizey_abs = sizey.abs();

        /* 
         * it is the same with position but instead of scale*parent_position it is scale*(parent_position - 1)
         * why? because we want a scale of 1 to put the top-left pixel of a given area in the bottom right most pixel INSIDE the parent area
         * if we use parent_position then a position of (x_scale: 1, y_scale: 1) puts the area right OUTSIDE the parent area
         * ok but why ask for that behavior?
         * well, its it made sense that (x_scale: 1, y_scale: 1) with anchor (1, 1) puts the bottom right area inside the parent area at the
         * bottom right
         */
        let posx = parent.position.x as f32 + ((parent.size.x).saturating_sub(1) as f32)*self.position.x.scale + self.position.x.offset as f32;
        let posy = parent.position.y as f32 + ((parent.size.y).saturating_sub(1) as f32)*self.position.y.scale + self.position.y.offset as f32;

        // copysign of size because that determines the direction of translation of the top-left area due to the anchor
        let anchorx = (sizex_abs - 1.0).max(0.0).copysign(sizex)*self.anchor.x;
        let anchory = (sizey_abs - 1.0).max(0.0).copysign(sizey)*self.anchor.y;

        AbsoluteSpace {
            size: Size2D {
                x: u16::try_float(sizex_abs.round()).unwrap(),
                y: u16::try_float(sizey_abs.round()).unwrap()
            },

            // negative size is coded as positive size but translated by negative size amount (gives the illusion of a flip)
            // anchor is coded as translating the top-left area to its new position due to do the anchor
            position: Point2D {
                x: i16::try_float((if sizex < 0.0 { posx + sizex } else { posx }).round() - anchorx.round()).unwrap(),
                y: i16::try_float((if sizey < 0.0 { posy + sizey } else { posy }).round() - anchory.round()).unwrap()
            }
        }
    }

    pub fn center(self) -> Space {
        Space {
            size: self.size,
            position: UDim2::CENTER,
            anchor: Float2D::CENTER
        }
    }
}

impl Lerp for Space {
    fn lerp(self, to: Self, alpha: f32) -> Self {
        Space { 
            size: self.size.lerp(to.size, alpha), 
            position: self.position.lerp(to.position, alpha),
            anchor: self.anchor.lerp(to.anchor, alpha) 
        }
    }
}

/* 
 * Needed float to u16, i16 conversion in Space::to_absolute_space method 
 * Converts f32 to u16, checks if same u16 converted to f32 == original f32, otherwise panic
 * Using this over "as" because I would like the application to panic if overflow occurs in release or if f32 is not rounded
 */
trait FloatTryFrom<T> {
    type Error;

    fn try_float(value: T) -> Result<Self, Self::Error>
        where Self: std::marker::Sized;
}

impl FloatTryFrom<f32> for u16 {
    type Error = String;

    fn try_float(x: f32) -> Result<Self, Self::Error> {
        let y = x as u16;
        if y as f32 == x {
            Ok(y)
        } else {
            Err(format!("{} {} could not be converted to {}", std::any::type_name::<f32>(), x, std::any::type_name::<u16>()))
        }
    }
}

impl FloatTryFrom<f32> for i16 {
    type Error = String;

    fn try_float(x: f32) -> Result<Self, Self::Error> {
        let y = x as i16;
        if y as f32 == x {
            Ok(y)
        } else {
            Err(format!("{} {} could not be converted to {}", std::any::type_name::<f32>(), x, std::any::type_name::<i16>()))
        }
    }
}