use std::cmp::{max, min};
use std::ops::Add;
use tui::layout::Rect;

pub trait Lerp {
    fn lerp(self, to: Self, alpha: f32) -> Self;
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Default)]
pub struct Point2D {
    pub x: i16,
    pub y: i16
}

impl Point2D {
    pub const fn new(x: i16, y: i16) -> Self {
        Point2D { x, y }
    }
}

impl Add<Point2D> for Point2D {
    type Output = Point2D;
    fn add(self, rhs: Point2D) -> Point2D {
        Point2D { 
            x: self.x.checked_add(rhs.x).unwrap(), 
            y: self.y.checked_add(rhs.y).unwrap()
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

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Default)]
pub struct Size2D {
    pub x: u16,
    pub y: u16
}

impl Size2D {
    pub const fn new(x: u16, y: u16) -> Self {
        Size2D { x, y }
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

impl Lerp for Size2D {
    fn lerp(self, to: Self, alpha: f32) -> Self {
        Size2D { 
            x: self.x + ((to.x as f32 - self.x as f32)*alpha).round() as u16,
            y: self.y + ((to.x as f32 - self.x as f32)*alpha).round() as u16
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Scale2D {
    pub x: f32,
    pub y: f32
}

impl Scale2D {
    pub const CENTER: Scale2D = Scale2D::new(0.5, 0.5);
    pub const fn new(x: f32, y: f32) -> Self {
        Scale2D { x, y }
    }
}

impl Lerp for Scale2D {
    fn lerp(self, to: Self, alpha: f32) -> Self {
        Scale2D { 
            x: self.x + (to.x - self.x)*alpha, 
            y: self.y + (to.y - self.y)*alpha
        }
    }
}

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

impl Lerp for UDim2 {
    fn lerp(self, to: Self, alpha: f32) -> Self {
        UDim2 { 
            x: self.x.lerp(to.x, alpha), 
            y: self.y.lerp(to.y, alpha) 
        }
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Space {
    pub size: UDim2,
    pub position: UDim2,
    pub anchor: Scale2D
}

impl Space {

    pub const FULL: Space = Space::sized(UDim2::from_scale(1.0, 1.0));

    pub const fn new(size: UDim2, position: UDim2, anchor: Scale2D) -> Self {
        Space { size, position, anchor }
    }

    pub const fn sized(size: UDim2) -> Self {
        Space::new(size, UDim2::new(0.0, 0, 0.0, 0), Scale2D::new(0.0, 0.0))
    }

    pub fn to_absolute_space(self, screen: AbsoluteSpace) -> AbsoluteSpace {
        let sizex = (screen.size.x as f32)*self.size.x.scale + self.size.x.offset as f32;
        let sizey = (screen.size.y as f32)*self.size.y.scale + self.size.y.offset as f32;
        let sizex_abs = sizex.abs();
        let sizey_abs = sizey.abs();

        let posx = screen.position.x as f32 + ((screen.size.x).saturating_sub(1) as f32)*self.position.x.scale + self.position.x.offset as f32;
        let posy = screen.position.y as f32 + ((screen.size.y).saturating_sub(1) as f32)*self.position.y.scale + self.position.y.offset as f32;
        let anchorx = (sizex_abs - 1.0).max(0.0).copysign(sizex)*self.anchor.x;
        let anchory = (sizey_abs - 1.0).max(0.0).copysign(sizey)*self.anchor.y;

        AbsoluteSpace {
            size: Size2D {
                x: u16::try_float(sizex_abs.round()).unwrap(),
                y: u16::try_float(sizey_abs.round()).unwrap()
            },
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
            anchor: Scale2D::CENTER
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