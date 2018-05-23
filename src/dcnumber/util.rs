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

pub fn carrying_map<I: IntoIterator, F>(iter: I, f: F, last_value: I::Item) -> CarryingMap<<I as IntoIterator>::IntoIter, F, I::Item>
    where
        F: Fn(bool, I::Item) -> (bool, I::Item)
{
    let iterator = iter.into_iter();
    CarryingMap { iter: iterator, f, last_value, carry: false }
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
}

