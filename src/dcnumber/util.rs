use std::fmt::Debug;

pub trait CarryingIterator {
    type Item;

    fn has_carry(&self) -> bool;
    fn set_carry(&mut self, carry: bool);
    fn carrying_next(&mut self) -> (bool, Option<Self::Item>);

    #[inline]
    fn with_carry(mut self, carry: bool) -> Self where
        Self: Sized
    {
        self.set_carry(carry);
        self
    }

    #[inline]
    fn carrying_map<B, F>(self, f: F) -> CarryingMap<Self, F>
        where
            Self: Sized,
            F: Fn(bool, Self::Item) -> (bool, B) {
        CarryingMap { iter: self, f}
    }


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

pub trait DoubleEndedCarryingIterator: CarryingIterator {
    fn carrying_next_back(&mut self) -> (bool, Option<Self::Item>);

    fn has_back_carry(&self) -> bool {
       self.has_carry()
    }

    fn set_back_carry(&mut self, carry: bool) {
        self.set_carry(carry)
    }

    fn with_back_carry(self, carry:bool) -> Self where
        Self: Sized
    {
        self.with_carry(carry)
    }
}


#[derive(Debug, Clone)]
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

impl <I: DoubleEndedIterator> DoubleEndedCarryingIterator for CarryingIter<I>
    where I::Item: Debug
{
    fn carrying_next_back(&mut self) -> (bool, Option<Self::Item>) {
        // TODO we might want to have a back-carry
        let carry = self.carry;
        let x = self.iter.next_back();
        match &x {
            &None => eprintln!("carrying_next_back, None"),
            &Some(ref a) => eprintln!("carrying_next_back, Some({:?})", a),
        };
        (carry, x)
    }
}

#[derive(Debug, Clone)]
pub struct IteratorAdapter<CI, U> {
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
            (_carry, Some(v)) => {
                // TODO are we sure we do not need to handle the carry also here?
                Some(v)
            }
        }
    }
}

impl <DECI: DoubleEndedCarryingIterator> DoubleEndedIterator for IteratorAdapter<DECI, DECI::Item> where
    DECI::Item: Clone + Debug,
{
    fn next_back(&mut self) -> Option<<Self as Iterator>::Item> {
        let x = match self.iter.carrying_next_back() {
            (true, None) => {
                    eprintln!("Injecting last_item: {:?}", self.last_item);
                    self.iter.set_back_carry(false);
                    Some(self.last_item.clone())
                }
            (false, None) => None,
            (_carry, Some(v)) => Some(v)
        };
        eprintln!("DoubleEndedIterator next_back {:?}", x);

        x
    }
}



#[derive(Debug, Clone)]
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


#[derive(Debug, Clone)]
enum CarryingChainState {
    Start,
    Last,
}

#[derive(Debug)]
pub struct CarryingChain<A, B> {
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
    fn rev() {
        let v = vec![2u8, 3u8];
        let mut expected = vec![3u8, 2u8];

        let actual: Vec<u8> = carrying(v.clone().into_iter().rev())
            .to_iter(0u8).collect();
        assert_eq!(expected, actual)
    }

    #[test]
    fn rev_carry() {
        let v = vec![2u8, 3u8];
        let mut expected = vec![3u8, 2u8, 1u8];

        let actual: Vec<u8> = carrying(v.clone().into_iter().rev())
            .with_carry(true)
            .to_iter(1u8).collect();
        assert_eq!(expected, actual)
    }

    #[test]
    fn rev2() {
        let v = vec![2u8, 3u8];
        let expected = v.clone();
        let actual: Vec<u8> = carrying(v.clone().into_iter().rev())
            .to_iter(0u8).rev().collect();
        assert_eq!( expected, actual )
    }

    #[ignore]
    #[test]
    fn rev2_carry() {
        let v = vec![2u8, 3u8];
        let expected = vec![1u8, 2u8, 3u8];
        let base = carrying(v.into_iter().rev().inspect(
            |x| eprintln!("inspect {:?}", x)
        ))
            .with_back_carry(true)
            .to_iter(1u8);

        let mut manual_rev: Vec<u8> = base.clone().collect();
        manual_rev.reverse();
        assert_eq!(expected, manual_rev);

        eprintln!("base.rev().collect()");
        let actual: Vec<u8> = base.rev().collect();
        assert_eq!( expected, actual )
    }

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

    #[test]
    fn chain_carry_3_no_carry() {
        assert_eq!(
            vec![2u32, 3u32, 4u32],
            carrying(vec![2u32])
                .carrying_chain(carrying(vec![3u32]))
                .carrying_chain(carrying(vec![4u32]))
                .to_iter(5u32)
                .collect::<Vec<u32>>());
    }

    #[test]
    fn chain_carry_3_carry_1() {
        assert_eq!(
            vec![2u32, 3u32, 4u32, 5u32],
            carrying(vec![2u32])
                .with_carry(true)
                .carrying_chain(carrying(vec![3u32]))
                .carrying_chain(carrying(vec![4u32]))
                .to_iter(5u32)
                .collect::<Vec<u32>>());
    }

    #[test]
    fn chain_carry_3_carry_2() {
        assert_eq!(
            vec![2u32, 3u32, 4u32, 5u32],
            carrying(vec![2u32])
                .carrying_chain(carrying(vec![3u32]))
                .with_carry(true)
                .carrying_chain(carrying(vec![4u32]))
                .to_iter(5u32)
                .collect::<Vec<u32>>());
    }

    #[test]
    fn chain_carry_3_carry_3() {
        assert_eq!(
            vec![2u32, 3u32, 4u32, 5u32],
            carrying(vec![2u32])
                .carrying_chain(carrying(vec![3u32]))
                .carrying_chain(carrying(vec![4u32]))
                .with_carry(true)
                .to_iter(5u32)
                .collect::<Vec<u32>>());
    }
}

