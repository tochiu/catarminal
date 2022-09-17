use super::{
    draw::*, 
    world::*, 
    space::*
};

pub type MountId = u64;

#[derive(Copy, Clone, Debug)]
pub struct MountFinder {
    id: MountId
}

impl MountFinder {
    const MAX_DEPTH: u32 = MountId::BITS / u8::BITS;

    pub fn new(id: MountId) -> Self {
        MountFinder { id }
    }
    pub fn peek(&self) -> u8 {
        (self.id & u8::MAX as MountId) as u8
    }
    pub fn next(self) -> MountFinder {
        MountFinder { id: self.id >> u8::BITS }
    }
    pub fn push(self, index: u8) -> MountFinder {
        MountFinder { id: self.id | (index as MountId) << u8::BITS*self.depth() }
    }
    pub fn depth(&self) -> u32 {
        MountFinder::MAX_DEPTH - self.id.leading_zeros() / u8::BITS
    }
}

pub trait Mountable: Layoutable + std::fmt::Debug + AsTrait + 'static {

    // implement

    fn mount_ref(&self) -> &Mount;
    fn mount_mut(&mut self) -> &mut Mount;
    fn child_ref(&self, i: usize) -> Option<&dyn Mountable>;
    fn child_mut(&mut self, i: usize) -> Option<&mut dyn Mountable>;

    // event handlers

    #[allow(unused_variables)]
    fn on_mouse_input(&mut self, event: WorldInputEvent) -> bool { false }
    
    // required / utility

    fn mount(&mut self, mut mount: Mount) {
        let mut itr = self.child_iter_mut();
        while let Some(child) = itr.next() {
            child.mount(mount.fork());
        }

        *self.mount_mut() = mount;
    }

    fn child_iter(&self) -> MountChildIter {
        MountChildIter { 
            drawing: self.as_trait_ref(), 
            index: 0
        }
    }

    fn child_iter_mut(&mut self) -> MountChildIterMut {
        MountChildIterMut { 
            drawing: self.as_trait_mut(),
            index: 0
        }
    }

    fn find_descendant_ref(&self, finder: MountFinder) -> Option<&dyn Mountable> {
        let index = finder.peek();

        if index == 0 {
            Some(self.as_trait_ref())
        } else {
            let maybe_child = self.child_ref(index as usize - 1);
            if let Some(child) = maybe_child {
                child.find_descendant_ref(finder.next())
            } else {
                None
            }
        }
    }

    fn find_descendant_mut(&mut self, finder: MountFinder) -> Option<&mut dyn Mountable> {
        let index = finder.peek();

        if index == 0 {
            Some(self.as_trait_mut())
        } else {
            let maybe_child = self.child_mut(index as usize - 1);
            if let Some(child) = maybe_child {
                child.find_descendant_mut(finder.next())
            } else {
                None
            }
        }
    }

    fn layout(&mut self, layout: WorldLayout) {
        self.layout_children(layout);
    }

    fn layout_children(&mut self, layout: WorldLayout) {
        let canvas_space = layout.parent_full_space;
        self.layout_children_in(layout, self.layout_ref().space.to_absolute_space(canvas_space));
    }

    fn layout_children_in(&mut self, layout: WorldLayout, canvas_space: AbsoluteSpace) {
        let full_space = self.layout_ref().space.to_absolute_space(layout.parent_full_space);
        if !full_space.intersects(layout.parent_draw_space) {
            return
        }

        let draw_space = full_space.intersection(layout.parent_draw_space);

        let mut itr = self.child_iter_mut();
        while let Some(child) = itr.next() {
            child.layout(WorldLayout {
                id: child.mount_ref().id,
                calculated_space: child.layout_ref().space.to_absolute_space(canvas_space),
                parent_draw_space: draw_space,
                parent_full_space: canvas_space,
                input: layout.input
            });
        }
    }

    fn update_input(&mut self, layout: &mut WorldLayout, input_space: Space) {
        let full_space = self.layout_ref().space.to_absolute_space(layout.parent_full_space);
        if !full_space.intersects(layout.parent_draw_space) {
            return
        }

        let draw_space = full_space.intersection(layout.parent_draw_space);

        let input_full_space = input_space.to_absolute_space(full_space);
        if !input_full_space.intersects(draw_space) {
            return
        }

        layout.input.update(layout.id, input_full_space.intersection(draw_space));
    }
}

pub trait AsTrait {
    fn as_trait_ref(&self) -> &dyn Mountable;
    fn as_trait_mut(&mut self) -> &mut dyn Mountable;
}

impl<T: Mountable + Sized> AsTrait for T {
    fn as_trait_ref(&self) -> &dyn Mountable { self }
    fn as_trait_mut(&mut self) -> &mut dyn Mountable { self }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct Mount {
    pub id: MountId,
    pub children: u8
}

impl Mount {
    fn fork(&mut self) -> Mount {
        self.children += 1;
        Mount { 
            id: MountFinder::new(self.id).push(self.children).id, 
            children: 0
        }
    }
}

pub struct MountChildIter<'a> {
    drawing: &'a dyn Mountable,
    index: usize
}

impl<'a> CustomIterator for MountChildIter<'a> {
    type Item = RefFamily<dyn Mountable>;

    fn next<'s>(&'s mut self) -> Option<<Self::Item as FamilyLt<'s>>::Out> {
        let maybe_child = self.drawing.child_ref(self.index);
        self.index += 1;
        maybe_child
    }
}

pub struct MountChildIterMut<'a> {
    drawing: &'a mut dyn Mountable,
    index: usize
}

impl<'a> CustomIterator for MountChildIterMut<'a> {
    type Item = MutRefFamily<dyn Mountable>;

    fn next<'s>(&'s mut self) -> Option<<Self::Item as FamilyLt<'s>>::Out> {
        let maybe_child = self.drawing.child_mut(self.index);
        self.index += 1;
        maybe_child
    }
}

/* 
 * the main use of this iterator is to provide custom mutable iterators without dipping unto unsafe code or the nightly channel
 * 
 * modified implementation of *Workaround B* (Using HRTBs) from
 * http://lukaskalbertodt.github.io/2018/08/03/solving-the-generalized-streaming-iterator-problem-without-gats.html
 * apparently the never type is only available on nightly (its not necessary but its abscence forgoes some compiler optimizations) 
 * 
 * UPDATE: GATs stabilization PR has been merged to rust-lang/rust
 * https://github.com/rust-lang/rust/pull/96709#issuecomment-1245350608
 * Milestone set to Rust v1.65.0
 */

use std::marker::PhantomData;

// ---------------------------------------------------------------------------
// The pub trait that abstracts over all families/type conpub structors that have one 
// lifetime input parameter.
pub trait FamilyLt<'a> {
    type Out;
}


// ---------------------------------------------------------------------------
// First we define a family that maps one type to itself.
pub struct IdFamily<T>(PhantomData<T>);

// Here we define the actual lifetime to type function
impl<'a, T> FamilyLt<'a> for IdFamily<T> {
    type Out = T;
}


// ---------------------------------------------------------------------------
// Here we define two families for the reference types `&T` and `&mut T`. 
pub struct RefFamily<T: ?Sized>(PhantomData<T>);
impl<'a, T: 'a + ?Sized> FamilyLt<'a> for RefFamily<T> {
    type Out = &'a T;
}

pub struct MutRefFamily<T: ?Sized>(PhantomData<T>);
impl<'a, T: 'a + ?Sized> FamilyLt<'a> for MutRefFamily<T> {
    type Out = &'a mut T;
}


// ---------------------------------------------------------------------------
// Here we define a family for the `Result` type. As you can see, the type 
// parameters are families, too, to allow for arbitrary nesting. (we could
// have done that for the reference families, too, but it's not necessary for
// this example.)
pub struct ResultFamily<T, E>(PhantomData<T>, PhantomData<E>);
impl<'a, T: FamilyLt<'a>, E: FamilyLt<'a>> FamilyLt<'a> for ResultFamily<T, E> {
    type Out = Result<T::Out, E::Out>;
}

// ---------------------------------------------------------------------------
/// The iterator pub trait that uses the family pattern
pub trait CustomIterator {
    // This basically reads: `Item` is a function from any lifetime to a type
    type Item: for<'a> FamilyLt<'a>;
    
    // "Use" the lifetime to type function here
    fn next<'s>(&'s mut self) -> Option<<Self::Item as FamilyLt<'s>>::Out>;
}