// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

/// a element of a meet-semilattice: https://en.wikipedia.org/wiki/Semilattice
pub trait MeetSemilattice: Eq + Clone {
    /// meet operator ∧
    /// greatest lower bound
    /// similar to the minimum operator
    fn meet(self, rhs: Self) -> Self;
    /// *self = self.meet(rhs)
    fn meet_assign(&mut self, rhs: Self) {
        *self = self.clone().meet(rhs);
    }
}

/// a element of a join-semilattice: https://en.wikipedia.org/wiki/Semilattice
pub trait JoinSemilattice: Eq + Clone {
    /// join operator ∨
    /// least upper bound
    /// similar to the maximum operator
    fn join(self, rhs: Self) -> Self;
    /// *self = self.join(rhs)
    fn join_assign(&mut self, rhs: Self) {
        *self = self.clone().join(rhs);
    }
}

/// an ordered lattice
/// helper trait that defines all other applicable traits in terms of this one
pub trait OrderedLattice: Ord + Clone {}

impl<T: OrderedLattice> MeetSemilattice for T {
    fn meet(self, rhs: Self) -> Self {
        self.min(rhs)
    }
}

impl<T: OrderedLattice> JoinSemilattice for T {
    fn join(self, rhs: Self) -> Self {
        self.max(rhs)
    }
}

/// an bounded ordered lattice
/// helper trait that defines all other applicable traits in terms of this one
pub trait BoundedOrderedLattice: Ord + Clone {
    fn min_value() -> Self;
    fn max_value() -> Self;
}

impl<T: BoundedOrderedLattice> OrderedLattice for T {}

impl<T: BoundedOrderedLattice> BottomBoundedLattice for T {
    fn bottom() -> Self {
        T::min_value()
    }
}

impl<T: BoundedOrderedLattice> TopBoundedLattice for T {
    fn top() -> Self {
        T::max_value()
    }
}

/// a element of a lattice: https://en.wikipedia.org/wiki/Lattice_%28order%29
pub trait Lattice: MeetSemilattice + JoinSemilattice {}

impl<T: MeetSemilattice + JoinSemilattice> Lattice for T {}

/// a element of a bottom-bounded lattice: https://en.wikipedia.org/wiki/Lattice_%28order%29
pub trait BottomBoundedLattice: Lattice {
    /// least element
    fn bottom() -> Self;
}

/// a element of a top-bounded lattice: https://en.wikipedia.org/wiki/Lattice_%28order%29
pub trait TopBoundedLattice: Lattice {
    /// greatest element
    fn top() -> Self;
}

/// a element of a bounded lattice: https://en.wikipedia.org/wiki/Lattice_%28order%29
pub trait BoundedLattice: BottomBoundedLattice + TopBoundedLattice {}

impl<T: BottomBoundedLattice + TopBoundedLattice> BoundedLattice for T {}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash, Default)]
#[repr(transparent)]
pub struct Reverse<T>(pub T);

impl<T: JoinSemilattice> MeetSemilattice for Reverse<T> {
    fn meet(self, rhs: Self) -> Self {
        Reverse(self.0.join(rhs.0))
    }
}

impl<T: MeetSemilattice> JoinSemilattice for Reverse<T> {
    fn join(self, rhs: Self) -> Self {
        Reverse(self.0.meet(rhs.0))
    }
}

impl<T: BottomBoundedLattice> TopBoundedLattice for Reverse<T> {
    fn top() -> Self {
        Reverse(T::bottom())
    }
}

impl<T: TopBoundedLattice> BottomBoundedLattice for Reverse<T> {
    fn bottom() -> Self {
        Reverse(T::top())
    }
}
