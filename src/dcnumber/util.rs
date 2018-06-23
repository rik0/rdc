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

}

