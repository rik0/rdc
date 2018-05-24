use std::iter::Chain;

#[derive(Debug)]
pub struct CarryingMap<I, F, B>
{
    iter: I,
    f: F,
    last_value: B,
    carry: bool,
}

impl<I: Iterator, F> Iterator for CarryingMap<I, F, I::Item>
    where
        F: Fn(bool, I::Item) -> (bool, I::Item),
        I::Item: Clone,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<I::Item> {
        match self.iter.next().map(|x| (self.f)(self.carry, x)) {
            None => {
                if self.carry {
                    self.carry = false;
                    Some(self.last_value.clone())
                } else {
                    None
                }
            }
            Some((new_carry, value)) => {
                self.carry = new_carry;
                Some(value)
            }
        }
    }
}

struct AsPairs<T> {
    it: T
}

impl<T> AsRef<T> for AsPairs<T> {
    fn as_ref(&self) -> &T {
        return &self.it;
    }
}

impl<T> AsMut<T> for AsPairs<T> {
    fn as_mut(&mut self) -> &mut T {
        return &mut self.it;
    }
}

impl<T> From<T> for AsPairs<T> {
    fn from(it: T) -> Self {
        AsPairs { it }
    }
}

impl<I: Iterator, F> Iterator for AsPairs<CarryingMap<I, F, I::Item>> where
    F: Fn(bool, I::Item) -> (bool, I::Item),
    I::Item: Clone,
{
    type Item = (bool, I::Item);

    fn next(&mut self) -> Option<Self::Item> {
        let (carry, value) = self.it.next_carrying();
        value
            .map(|item| (carry, item))
    }
}

impl<I: Iterator, F> CarryingMap<I, F, I::Item> where
    F: Fn(bool, I::Item) -> (bool, I::Item),
    I::Item: Clone,
{
    pub fn chain_carrying<J: Iterator<Item=I::Item>, G>(self, other: CarryingMap<J, G, I::Item>) -> CarryingChain<I, J, F, G, I::Item> {
        CarryingChain{head: self, last: other, state: CarryingChainState::Start}
    }

    pub fn next_carrying(&mut self) -> (bool, Option<I::Item>) {
        match self.iter.next().map(|x| (self.f)(self.carry, x)) {
            None => {
                (self.carry, None)
            }
            Some((new_carry, value)) => {
                self.carry = new_carry;
                (new_carry, Some(value))
            }
        }
    }
}

pub fn carrying_map<I: IntoIterator, F>(iter: I, f: F, last_value: I::Item) -> CarryingMap<<I as IntoIterator>::IntoIter, F, I::Item>
    where
        F: Fn(bool, I::Item) -> (bool, I::Item)
{
    let iterator = iter.into_iter();
    CarryingMap { iter: iterator, f, last_value, carry: false }
}

#[derive(Debug)]
enum CarryingChainState {
    Start,
    Last,
}

#[derive(Debug)]
struct CarryingChain<I: Iterator, J: Iterator, F, G, U> {
    // TODO make it work with a sequence of chains maybe? for now we need just the one with two
    // TODO consider creating a CarryingIterator trait instead
    head: CarryingMap<I,F, U>,
    last: CarryingMap<J, G, U>,
    state: CarryingChainState,
}


impl<I:Iterator, J: Iterator<Item=I::Item>, F, G> Iterator for CarryingChain<I, J, G, F, I::Item>  where
    F: Fn(bool, I::Item) -> (bool, I::Item),
    G: Fn(bool, I::Item) -> (bool, I::Item),
    I::Item: Clone,
{
    type Item = I::Item;


    fn next(&mut self) -> Option<I::Item> {
        use std::iter::Iterator;
        // TODO possibly we want to get some because we need to detect that the first iterator is going
        // to release the carry (or find another way... maybe only if carry is active? if not,
        // it does not matter)

        // 1. get an item with carry and put it somewhere, even locally
        // 2. if there is carry, we know it can be the digit carried over
        // 3. get the item after this one: if it is None, it means that this item
        //

        match self.state {
            CarryingChainState::Start => {
                match self.head.next_carrying() {
                    (carry, None) => {
                        self.last.carry = carry;
                        self.state = CarryingChainState::Last;
                        self.last.next()
                    }
                    (_, Some(v)) => {
                        Some(v)
                    }
                }
            }
            CarryingChainState::Last => {
                self.last.next()
            }
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
        let one: u32 = 0 + 1;
        let v = vec![1u32];
        let x: Vec<u32> = carrying_map(v.clone().into_iter(), |carry, x| (carry, x), one).collect();

        assert_eq!(v, x);
    }

    #[test]
    fn carry() {
        use std::iter::Iterator;
        let one: u32 = 0 + 1;
        let mut v = vec![2u32];
        let x: Vec<u32> = carrying_map(v.clone().into_iter(), |carry, x| (true, x), one).collect();

        v.push(1);

        assert_eq!(v, x);
    }

    #[test]
    fn chain_carry_carry_across() {
        let mut v = vec![2u32];
        let mut u = vec![3u32];
        let ch1 = carrying_map(v.clone().into_iter(), |carry, x| (true, x), 1u32);
        let ch2 = carrying_map(u.clone().into_iter(), |carry, x| (true, x), 4u32);
        let chain = ch1.chain_carrying(ch2);
        let actual: Vec<u32> = chain.collect();

        assert_eq!(vec![2u32, 3u32, 4u32], actual);
    }

    #[test]
    fn chain_carry_carry_across_and_suppress() {
        let mut v = vec![2u32];
        let mut u = vec![3u32];
        let ch1 = carrying_map(v.clone().into_iter(), |carry, x| (true, x), 1u32);
        let ch2 = carrying_map(u.clone().into_iter(), |carry, x| (false, x), 4u32);
        let chain = ch1.chain_carrying(ch2);
        let actual: Vec<u32> = chain.collect();

        assert_eq!(vec![2u32, 3u32], actual);
    }

    #[test]
    fn chain_carry_no_carry() {
        let mut v = vec![2u32];
        let mut u = vec![3u32];
        let ch1 = carrying_map(v.clone().into_iter(), |carry, x| (false, x), 1u32);
        let ch2 = carrying_map(u.clone().into_iter(), |carry, x| (carry, x), 4u32);
        let chain = ch1.chain_carrying(ch2);
        let actual: Vec<u32> = chain.collect();

        assert_eq!(vec![2u32, 3u32], actual);
    }

    #[test]
    fn chain_carry_carry() {
        let mut v = vec![2u32];
        let mut u = vec![3u32];
        let ch1 = carrying_map(v.clone().into_iter(), |carry, x| (true, x), 1u32);
        let ch2 = carrying_map(u.clone().into_iter(), |carry, x| (carry, x), 4u32);
        let chain = ch1.chain_carrying(ch2);
        let actual: Vec<u32> = chain.collect();

        assert_eq!(vec![2u32, 3u32, 4u32], actual);
    }

    #[test]
    fn chain_carry_carry_across2() {
        let mut v = vec![2u32];
        let mut u = vec![3u32];
        let ch1 = carrying_map(v.clone().into_iter(), |carry, x| (false, x), 1u32);
        let ch2 = carrying_map(u.clone().into_iter(), |carry, x| (true, x), 4u32);
        let chain = ch1.chain_carrying(ch2);
        let actual: Vec<u32> = chain.collect();

        assert_eq!(vec![2u32, 3u32, 4u32], actual);
    }
}

