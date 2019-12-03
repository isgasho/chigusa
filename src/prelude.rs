use crate::c0::lexer::TokenType;
use std::str::{Chars, FromStr};
use std::{
    cell::{Ref, RefCell, RefMut},
    cmp::PartialOrd,
    fmt,
    fmt::Display,
    fmt::Formatter,
    hash::Hash,
    ops::Try,
    rc::{Rc, Weak},
    string::String,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Pos {
    pub ln: usize,
    pub pos: usize,
    pub index: usize,
}

impl Pos {
    pub fn new(ln: usize, pos: usize, index: usize) -> Pos {
        Pos { ln, pos, index }
    }

    pub fn zero() -> Pos {
        Pos {
            ln: 0,
            pos: 0,
            index: 0,
        }
    }

    #[must_use]
    #[inline]
    pub fn bump(mut self) -> Pos {
        self.index += 1;
        self
    }

    #[must_use]
    #[inline]
    pub fn inc(mut self) -> Pos {
        self.pos += 1;
        self.index += 1;
        self
    }

    #[must_use]
    #[inline]
    pub fn lf(mut self) -> Pos {
        self.pos = 0;
        self.ln += 1;
        self.index += 1;
        self
    }

    #[must_use]
    pub fn map_inc(mut self, pos_offset: isize, ln_offset: isize, index_offset: isize) -> Pos {
        self.pos = if pos_offset >= 0 {
            self.pos + pos_offset as usize
        } else {
            self.pos - (-pos_offset) as usize
        };
        self.ln = if ln_offset >= 0 {
            self.ln + ln_offset as usize
        } else {
            self.ln - (-ln_offset) as usize
        };
        self.index = if index_offset >= 0 {
            self.index + index_offset as usize
        } else {
            self.index - (-index_offset) as usize
        };
        self
    }

    #[inline]
    pub fn inc_self(&mut self) {
        self.pos += 1;
        self.index += 1;
    }

    #[inline]
    pub fn lf_self(&mut self) {
        self.pos = 0;
        self.ln += 1;
        self.index += 1;
    }

    #[inline]
    pub fn bump_self(&mut self) {
        self.index += 1;
    }
}

impl Display for Pos {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Pos(idx {}, ln {} col {})",
            self.index, self.ln, self.pos
        )
    }
}

impl PartialOrd for Pos {
    fn partial_cmp(&self, other: &Pos) -> Option<std::cmp::Ordering> {
        Some(self.index.cmp(&other.index))
    }
}

impl Ord for Pos {
    fn cmp(&self, other: &Pos) -> std::cmp::Ordering {
        self.index.cmp(&other.index)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Span {
    pub start: Pos,
    pub end: Pos,
}

impl Span {
    pub fn from(start: Pos, end: Pos) -> Span {
        assert!(start <= end);
        Span { start, end }
    }

    pub fn point(pos: Pos) -> Span {
        Span::from(pos, pos)
    }

    pub fn zero() -> Span {
        Span::from(Pos::zero(), Pos::zero())
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}->{}", self.start, self.end)
    }
}

impl std::ops::Add for Span {
    type Output = Span;

    /// An add between spans is combining them together, and filling the center
    /// if any hollow part appears. Adding spans only preserves the ordination
    /// between indexes, so only adding spans from the same string makes sense.
    ///
    /// # Example
    ///
    /// ```
    /// # use chigusa::c0::infra::{Pos, Span};
    /// let lhs = Span {
    ///     start: Pos {
    ///         ln: 1,
    ///         pos: 1,
    ///         index: 0,
    ///     },
    ///     end: Pos {
    ///         ln: 1,
    ///         pos: 3,
    ///         index: 2,
    ///     }
    /// };
    ///
    /// let rhs = Span {
    ///     start: Pos {
    ///         ln: 1,
    ///         pos: 5,
    ///         index: 4,
    ///     },
    ///     end: Pos {
    ///         ln: 1,
    ///         pos: 9,
    ///         index: 8,
    ///     }
    /// };
    ///
    /// assert_eq!(lhs + rhs, Span {
    ///     start: Pos {
    ///         ln: 1,
    ///         pos: 1,
    ///         index: 0,
    ///     },
    ///     end: Pos {
    ///         ln: 1,
    ///         pos: 9,
    ///         index: 8,
    ///     }
    /// });
    /// ```
    fn add(self, rhs: Span) -> Self::Output {
        let smaller_start = std::cmp::min(self.start, rhs.start);
        let larger_end = std::cmp::max(self.end, rhs.end);
        Span {
            start: smaller_start,
            end: larger_end,
        }
    }
}
/*
    // Subtracting spans make no sense for now

    impl std::ops::Sub for Span{
        type Output = Span;
        fn sub(self, rhs: Span)->Self::Output{
            if self.start==rhs.start{
                Span{
                    start: rhs.end,
                    end: self.end
                }
            } else {

            }
        }
    }
*/

#[derive(Eq, PartialEq, Debug)]
pub struct Ptr<T>(Rc<RefCell<T>>);

impl<T> Ptr<T> {
    pub fn new(val: T) -> Ptr<T> {
        Ptr(Rc::new(RefCell::new(val)))
    }

    pub fn borrow(&self) -> Ref<T> {
        self.0.borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<T> {
        self.0.borrow_mut()
    }

    pub fn downgrade(self) -> Weak<RefCell<T>> {
        std::rc::Rc::downgrade(&self.0)
    }
}

impl<T> Clone for Ptr<T> {
    fn clone(&self) -> Self {
        Ptr(Rc::clone(&self.0))
    }
}
impl<T> fmt::Display for Ptr<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let borrow = self.borrow();
        write!(f, "{}", borrow)
    }
}
pub trait IntoPtr<T> {
    fn into_ptr(self) -> Ptr<T>;
}

impl<T> IntoPtr<T> for Rc<RefCell<T>> {
    fn into_ptr(self) -> Ptr<T> {
        Ptr(self)
    }
}

pub trait AstNode {
    fn span(&self) -> Span;
    // fn return_type(&self, scope: &super::ast::Scope) -> Option<&str>;
}

#[inline]
pub fn variant_eq<T>(a: &T, b: &T) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}

pub enum LoopCtrl<T> {
    Stop(T),
    Continue,
}

use LoopCtrl::*;

impl<T> LoopCtrl<T> {
    pub fn unwrap(self) -> T {
        match self {
            LoopCtrl::Stop(x) => x,
            _ => panic!("Cannot unwrap a LoopCtrl with Continue statement"),
        }
    }

    pub fn is_continue(&self) -> bool {
        match self {
            LoopCtrl::Continue => true,
            _ => false,
        }
    }
}

impl<T> From<Option<T>> for LoopCtrl<T> {
    fn from(other: Option<T>) -> LoopCtrl<T> {
        match other {
            Some(v) => Stop(v),
            None => Continue,
        }
    }
}

impl<T> From<Result<(), T>> for LoopCtrl<T> {
    fn from(other: Result<(), T>) -> LoopCtrl<T> {
        match other {
            Ok(_) => Continue,
            Err(e) => Stop(e),
        }
    }
}

impl<T> Into<Result<(), T>> for LoopCtrl<T> {
    fn into(self) -> Result<(), T> {
        match self {
            Continue => Ok(()),
            Stop(x) => Err(x),
        }
    }
}

impl<T> Try for LoopCtrl<T> {
    type Ok = ();
    type Error = T;
    fn into_result(self) -> Result<Self::Ok, Self::Error> {
        self.into()
    }
    fn from_error(v: Self::Error) -> Self {
        Stop(v)
    }
    fn from_ok(v: Self::Ok) -> Self {
        Continue
    }
}

#[inline]
pub fn loop_while<F, T>(mut f: F) -> T
where
    F: FnMut() -> LoopCtrl<T>,
{
    let mut x: LoopCtrl<T> = Continue;
    while x.is_continue() {
        x = f();
    }
    // the following unwrap CANNOT panic because x is garanteed to be Some.
    x.unwrap()
}

pub fn loop_while_check<F, T, E>(mut body: F) -> Result<T, E>
where
    F: FnMut() -> Result<LoopCtrl<T>, E>,
{
    let mut x: LoopCtrl<T> = body()?;
    while x.is_continue() {
        x = body()?;
    }
    Ok(x.unwrap())
}

#[macro_export]
macro_rules! set {
    ( $( $x:expr ),* ) => {  // Match zero or more comma delimited items
        {
            let mut temp_set = HashSet::new();  // Create a mutable HashSet
            $(
                temp_set.insert($x); // Insert each item matched into the HashSet
            )*
            temp_set // Return the populated HashSet
        }
    };
}
