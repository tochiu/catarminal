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