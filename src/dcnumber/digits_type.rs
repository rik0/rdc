use std::rc::Rc;
use std::ops::Index;
use std::ops::RangeTo;
use std::ops::RangeFrom;
use std::ops::Range;

#[derive(Debug)]
pub enum DigitsType {
    Ref(&'static [u8]),
    V(Rc<Vec<u8>>)
}

impl DigitsType {
    #[inline]
    fn get_ref(&self) -> &[u8] {
        match self {
            DigitsType::Ref(r) => r,
            DigitsType::V(ref v) => v.as_ref(),
        }

    }

    pub fn into_vec(self) -> Vec<u8> {
        match self {
            DigitsType::Ref(r) => Vec::from(r),
            DigitsType::V(v) => Rc::try_unwrap(v)
                .unwrap_or_else(|rc| rc.as_ref().clone())
        }
    }

    pub fn len(&self) -> usize {
        self.get_ref().len()
    }
    pub fn holds_memory(&self) -> bool {
        match self {
            DigitsType::Ref(_r) => false,
            DigitsType::V(rc) => Rc::strong_count(rc) == 1
        }
    }
}


impl From<&'static [u8]> for DigitsType {
    fn from(v: &'static [u8]) -> Self {
        DigitsType::Ref(v)
    }
}

impl From<Vec<u8>> for DigitsType
{
    fn from(v: Vec<u8>) -> Self {
        DigitsType::V(Rc::from(v))
    }
}


impl AsRef<[u8]> for DigitsType {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.get_ref()
    }
}

impl Clone for DigitsType {
    fn clone(&self) -> Self {
        match self {
            DigitsType::Ref(r) => DigitsType::Ref(r),
            DigitsType::V(v) => DigitsType::V(v.clone()),
        }
    }
}

impl Index<usize> for DigitsType {
    type Output = u8;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.as_ref()[index]
    }
}


impl Index<RangeTo<usize>> for DigitsType {
    type Output = [u8];

    fn index(&self, index: RangeTo<usize>) -> &[u8] {
        &self.as_ref()[index]
    }
}

impl Index<RangeFrom<usize>> for DigitsType {
    type Output = [u8];

    fn index(&self, index: RangeFrom<usize>) -> &[u8] {
        &self.as_ref()[index]
    }
}


impl Index<Range<usize>> for DigitsType {
    type Output = [u8];

    fn index(&self, index: Range<usize>) -> &[u8] {
        &self.as_ref()[index]
    }
}

macro_rules! digits {
    ( $( $digits:expr ),* ) => ( DigitsType::from([ $( $digits as u8), * ].as_ref()) )

}
