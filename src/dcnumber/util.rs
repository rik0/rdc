

pub trait CarryingIterator {
    type Item;

    fn has_carry(&self) -> bool;
    fn set_carry(&mut self, carry: bool);
    fn carrying_next(&mut self) -> (bool, Option<Self::Item>);

    #[inline]
    fn carrying_map<B, F>(self, f: F) -> CarryingMap<Self, F>
        where
            Self: Sized,
            F: Fn(bool, Self::Item) -> (bool, B) {
        let carry = self.has_carry();
        CarryingMap { iter: self, f}
    }

//    #[inline]
//    fn carrying_chain<U>(self, other: U) -> CarryingChain<Self, U::IntoIter> where
//        Self: Sized, U: IntoIterator<Item=Self::Item>,
//    {
//        CarryingIterator::carrying_chain2(self, carrying(other))
//    }

    #[inline]
    fn carrying_chain<CI>(self, other: CI) -> CarryingChain<Self, CI> where
        CI: CarryingIterator<Item=Self::Item>,
        Self: Sized
    {
        CarryingChain{head: self, last: other, state: CarryingChainState::Start}
    }


    #[inline]
    fn to_iter(self, last_item: Self::Item) -> IteratorAdapter<Self, Self::Item> where
        Self: Sized,
        Self::Item: Clone
    {
        IteratorAdapter{iter: self, last_item}
    }

}

#[derive(Debug)]
pub struct CarryingIter<I> {
    iter: I,
    carry: bool,
}

#[inline]
pub fn carrying<I: IntoIterator>(iter: I) -> CarryingIter<I::IntoIter> {
    let iter = iter.into_iter();
    CarryingIter{iter, carry: false}
}

impl <I: Iterator> CarryingIterator for CarryingIter<I> {
    type Item = I::Item;

    #[inline]
    fn has_carry(&self) -> bool {
        self.carry
    }

    #[inline]
    fn set_carry(&mut self, carry: bool) {
        self.carry = carry
    }

    #[inline]
    fn carrying_next(&mut self) -> (bool, Option<Self::Item>) {
        let carry = self.carry;
        (carry, self.iter.next())
    }
}

struct IteratorAdapter<CI, U> {
    iter: CI,
    last_item: U,
}

impl <CI: CarryingIterator> Iterator for IteratorAdapter<CI, CI::Item> where
    CI::Item: Clone
{
    type Item = CI::Item;

    #[inline]
    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        match self.iter.carrying_next() {
            (carry, None) => {
                if carry {
                    self.iter.set_carry(false);
                    Some(self.last_item.clone())
                } else {
                    None
                }
            }
            (carry, Some(v)) => {
                // TODO are we sure we do not need to handle the carry also here?
                Some(v)
            }
        }
    }
}
//
//impl <CI: CarryingIterator> IntoIterator for IteratorAdapter<CI, CI::Item> where
//    CI: Sized,
//    CI::Item: Clone
//{
//    type Item = CI::Item;
//    type IntoIter = Self;
//
//    fn into_iter(self) -> <Self as IntoIterator>::IntoIter {
//        unimplemented!()
//    }
//}


#[derive(Debug)]
pub struct CarryingMap<I, F>
{
    iter: I,
    f: F,
}

impl <B, CI:CarryingIterator, F> CarryingIterator for CarryingMap<CI, F>  where
    F: Fn(bool, CI::Item) -> (bool, B),
{
    type Item = B;

    #[inline]
    fn has_carry(&self) -> bool {
        self.iter.has_carry()
    }

    #[inline]
    fn set_carry(&mut self, carry: bool) {
        self.iter.set_carry(carry)
    }

    #[inline]
    fn carrying_next(&mut self) -> (bool, Option<Self::Item>) {
        match self.iter.carrying_next() {
            (carry, None) => (carry, None),
            (carry, Some(v)) => {
                let (new_carry, new_v) = (self.f)(carry, v);
                self.set_carry(new_carry);
                (new_carry, Some(new_v))
            }
        }
    }
}


#[derive(Debug)]
enum CarryingChainState {
    Start,
    Last,
}

#[derive(Debug)]
pub struct CarryingChain<A, B> {
    // TODO make it work with a sequence of chains maybe? for now we need just the one with two
    // TODO consider creating a CarryingIterator trait instead
    head: A,
    last: B,
    state: CarryingChainState,
}



impl <A, B> CarryingIterator for CarryingChain<A, B> where
    A: CarryingIterator,
    B: CarryingIterator<Item=A::Item>
{

    type Item = A::Item;

    #[inline]
    fn has_carry(&self) -> bool {
        match self.state {
            CarryingChainState::Start => self.head.has_carry(),
            CarryingChainState::Last => self.last.has_carry(),
        }
    }

    #[inline]
    fn set_carry(&mut self, carry: bool) {
        match self.state {
            CarryingChainState::Start => self.head.set_carry(carry),
            CarryingChainState::Last => self.last.set_carry(carry),
        }
    }

    #[inline]
    fn carrying_next(&mut self) -> (bool, Option<<Self as CarryingIterator>::Item>) {
        match self.state {
            CarryingChainState::Start => {
                match self.head.carrying_next() {
                    (carry, None) => {
                        self.last.set_carry(carry);
                        self.state = CarryingChainState::Last;
                        self.last.carrying_next()
                    }
                    (carry, Some(v)) => {
                        (carry, Some(v))
                    }
                }
            },
            CarryingChainState::Last => self.last.carrying_next(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::iter::Iterator;

    #[test]
    fn no_carry() {
        use std::iter::Iterator;
        let v = vec![1u32];
        let x: Vec<u32> = carrying(v.clone())
            .carrying_map(|carry, x| (carry, x))
            .to_iter(1u32)
            .collect();

        assert_eq!(v, x);
    }

    #[test]
    fn carry() {
        use std::iter::Iterator;
        let mut v = vec![2u32];
        let x: Vec<u32> = carrying(v.clone())
            .carrying_map(|carry, x| (true, x))
            .to_iter(1u32)
            .collect();

        v.push(1);

        assert_eq!(v, x);
    }

    #[test]
    fn chain_carry_carry_false() {
        let mut v = vec![2u32];
        let mut u = vec![3u32];

        let actual: Vec<u32> = carrying(v.into_iter())
            .carrying_chain(carrying(u.into_iter()))
            .to_iter(4u32)
            .collect();

        assert_eq!(vec![2u32, 3u32], actual);
    }

    #[test]
    fn chain_carry_carry_true() {
        let mut v = vec![2u32];
        let mut u = vec![3u32];

        let actual: Vec<u32> = carrying(v.into_iter())
            .carrying_chain(carrying(u.into_iter()))
            .carrying_map(|carry, v| (true, v))
            .to_iter(4u32)
            .collect();

        assert_eq!(vec![2u32, 3u32, 4u32], actual);
    }
}

