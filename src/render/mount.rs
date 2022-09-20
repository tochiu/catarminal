use super::{
    draw::*, 
    screen::*, 
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

pub trait MountableLayout: Layoutable + std::fmt::Debug + AsTrait + 'static {

    // implement

    fn mount_ref(&self) -> &Mount;
    fn mount_mut(&mut self) -> &mut Mount;
    fn child_ref(&self, i: usize) -> Option<&dyn MountableLayout>;
    fn child_mut(&mut self, i: usize) -> Option<&mut dyn MountableLayout>;

    // event handlers

    #[allow(unused_variables)]
    fn on_mouse_input(&mut self, event: ScreenInputEvent) -> bool { false }
    
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

    fn find_descendant_ref(&self, finder: MountFinder) -> Option<&dyn MountableLayout> {
        let index = finder.peek();

        if index == 0 {
            Some(self.as_trait_ref())
        } else {
            if let Some(child) = self.child_ref(index as usize - 1) {
                child.find_descendant_ref(finder.next())
            } else {
                None
            }
        }
    }

    fn find_descendant_mut(&mut self, finder: MountFinder) -> Option<&mut dyn MountableLayout> {
        let index = finder.peek();

        if index == 0 {
            Some(self.as_trait_mut())
        } else {
            if let Some(child) = self.child_mut(index as usize - 1) {
                child.find_descendant_mut(finder.next())
            } else {
                None
            }
        }
    }

    fn relayout(&mut self, relayout: ScreenRelayout) {
        self.relayout_children(relayout);
    }

    fn relayout_children(&mut self, relayout: ScreenRelayout) {
        let transformed_absolute_layout_space = self.to_absolute_layout_space(relayout.parent_absolute_layout_space);
        self.relayout_children_in(relayout, transformed_absolute_layout_space);
    }

    fn relayout_children_in(&mut self, relayout: ScreenRelayout, transformed_absolute_layout_space: AbsoluteSpace) {
        let absolute_layout_space = self.to_absolute_layout_space(relayout.parent_absolute_layout_space);
        if let Some(absolute_draw_space) = relayout.restrict_absolute_layout_space(absolute_layout_space) {
            let mut itr = self.child_iter_mut();
            while let Some(child) = itr.next() {
                child.relayout(ScreenRelayout {
                    id: child.mount_ref().id,
                    absolute_layout_space: child.to_absolute_layout_space(transformed_absolute_layout_space),
                    parent_absolute_draw_space: absolute_draw_space,
                    parent_absolute_layout_space: transformed_absolute_layout_space,
                    input: relayout.input
                });
            }
        }
    }

    fn relayout_input_space(&mut self, relayout: &mut ScreenRelayout, input_space: Space) {
        let absolute_layout_space = self.to_absolute_layout_space(relayout.parent_absolute_layout_space);
        if let Some(absolute_draw_space) = relayout.restrict_absolute_layout_space(absolute_layout_space) {
            if let Some(input_absolute_interactable_space) = 
                input_space
                    .to_absolute_space(absolute_layout_space)
                    .try_intersection(absolute_draw_space) 
            {
                relayout.input.update(relayout.id, input_absolute_interactable_space);
            }
        }
    }
}

pub trait AsTrait {
    fn as_trait_ref(&self) -> &dyn MountableLayout;
    fn as_trait_mut(&mut self) -> &mut dyn MountableLayout;
}

impl<T: MountableLayout + Sized> AsTrait for T {
    fn as_trait_ref(&self) -> &dyn MountableLayout { self }
    fn as_trait_mut(&mut self) -> &mut dyn MountableLayout { self }
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
    drawing: &'a dyn MountableLayout,
    index: usize
}

impl<'a> CustomIterator for MountChildIter<'a> {
    type Item = RefFamily<dyn MountableLayout>;

    fn next<'s>(&'s mut self) -> Option<<Self::Item as FamilyLt<'s>>::Out> {
        let maybe_child = self.drawing.child_ref(self.index);
        self.index += 1;
        maybe_child
    }
}

pub struct MountChildIterMut<'a> {
    drawing: &'a mut dyn MountableLayout,
    index: usize
}

impl<'a> CustomIterator for MountChildIterMut<'a> {
    type Item = MutRefFamily<dyn MountableLayout>;

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