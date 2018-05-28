

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
        CarryingMap { iter: self, f, carry }
    }

//    #[inline]
//    fn carrying_chain<U>(self, other: U) -> CarryingChain<Self, U::IntoIter> where
//        Self: Sized, U: IntoIterator<Item=Self::Item>,
//    {
//        CarryingIterator::chain(self, carrying(other))
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

impl <I: IntoIterator> From<I> for CarryingIter<I::IntoIter> {
    #[inline]
    fn from(iter: I) -> Self {
        carrying(iter)
    }
}

//impl <I: Iterator> Iterator for CarryingIter<I> {
//    type Item = I::Item;
//
//    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
//        self.iter.next()
//    }
//}

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
    carry: bool,
}

// TODO: what to do about the "last value"? I think this should be part of the "to normal iterator"
// conversion...

//impl<B, I: Iterator, F> Iterator for CarryingMap<I, F> where
//        F: Fn(bool, I::Item) -> (bool, B),
//        B: Clone,
//{
//    type Item = B;
//
//    fn next(&mut self) -> Option<B> {
//        match self.iter.next().map(|x| (self.f)(self.carry, x)) {
//            None => {
//                if self.carry {
//                    self.carry = false;
//                    Some(self.last_value.clone())
//                } else {
//                    None
//                }
//            }
//            Some((new_carry, value)) => {
//                self.carry = new_carry;
//                Some(value)
//            }
//        }
//    }
//}

impl <B, CI:CarryingIterator, F> CarryingIterator for CarryingMap<CI, F>  where
    F: Fn(bool, CI::Item) -> (bool, B),
{
    type Item = B;

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
            CarryingChainState::Start => self.head.carrying_next(),
            CarryingChainState::Last => self.last.carrying_next(),
        }
    }
}

//impl <A, B> Iterator for CarryingChain<A, B> where
//    A: CarryingIterator,
//    B: CarryingIterator<Item=A::Item>
//{
//    type Item = A::Item;
//
//    fn next(&mut self) -> Option<Self::Item> {
//        use std::iter::Iterator;
//
//        match self.state {
//            CarryingChainState::Start => {
//                match self.head.carrying_next() {
//                    (carry, None) => {
//                        self.last.set_carry(carry);
//                        self.state = CarryingChainState::Last;
//                        self.last.next()
//                    }
//                    (_, Some(v)) => {
//                        Some(v)
//                    }
//                }
//            }
//            CarryingChainState::Last => {
//                self.last.next()
//            }
//        }
//    }
//}

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

//    #[test]
//    fn chain_carry_carry_false() {
//        let mut v = vec![2u32];
//        let mut u = vec![3u32];
//
//        let actual: Vec<u32> = carrying(v.clone().into_iter())
//            .carrying_chain(u.clone().into_iter())
//            .to_iter(4u32)
//            .collect();
//
//        assert_eq!(vec![2u32, 3u32], actual);
//    }

//    #[test]
//    fn chain_carry_carry_across_and_suppress() {
//        let mut v = vec![2u32];
//        let mut u = vec![3u32];
//        let ch1 = carrying_map(v.clone().into_iter(), |carry, x| (true, x), 1u32);
//        let ch2 = carrying_map(u.clone().into_iter(), |carry, x| (false, x), 4u32);
//        let chain = ch1.carrying_chain(ch2);
//        let actual: Vec<u32> = chain.collect();
//
//        assert_eq!(vec![2u32, 3u32], actual);
//    }
//
//    #[test]
//    fn chain_carry_no_carry() {
//        let mut v = vec![2u32];
//        let mut u = vec![3u32];
//        let ch1 = carrying_map(v.clone().into_iter(), |carry, x| (false, x), 1u32);
//        let ch2 = carrying_map(u.clone().into_iter(), |carry, x| (carry, x), 4u32);
//        let chain = ch1.carrying_chain(ch2);
//        let actual: Vec<u32> = chain.collect();
//
//        assert_eq!(vec![2u32, 3u32], actual);
//    }
//
//    #[test]
//    fn chain_carry_carry() {
//        let mut v = vec![2u32];
//        let mut u = vec![3u32];
//        let ch1 = carrying_map(v.clone().into_iter(), |carry, x| (true, x), 1u32);
//        let ch2 = carrying_map(u.clone().into_iter(), |carry, x| (carry, x), 4u32);
//        let chain = ch1.carrying_chain(ch2);
//        let actual: Vec<u32> = chain.collect();
//
//        assert_eq!(vec![2u32, 3u32, 4u32], actual);
//    }
//
//    #[test]
//    fn chain_carry_carry_across2() {
//        let mut v = vec![2u32];
//        let mut u = vec![3u32];
//        let ch1 = carrying_map(v.clone().into_iter(), |carry, x| (false, x), 1u32);
//        let ch2 = carrying_map(u.clone().into_iter(), |carry, x| (true, x), 4u32);
//        let chain = ch1.carrying_chain(ch2);
//        let actual: Vec<u32> = chain.collect();
//
//        assert_eq!(vec![2u32, 3u32, 4u32], actual);
//    }
}

