use std::cmp::{Ordering, PartialEq, PartialOrd};
use std::fmt;

struct LogIndex {
    idx: usize,
    len: usize,
}

impl LogIndex {
    fn logical(&self) -> usize {
        self.idx
    }

    fn physical(&self) -> usize {
        self.idx % self.len
    }
}

impl fmt::Debug for LogIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "LogIndex {{ physical: {}, logical: {} }}",
            self.physical(),
            self.logical()
        )
    }
}

impl PartialEq for LogIndex {
    fn eq(&self, other: &Self) -> bool {
        self.logical() == other.logical()
    }
}

impl PartialOrd for LogIndex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.logical().cmp(&other.logical()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::{quickcheck, TestResult};

    quickcheck! {
        fn test_logindex(idx: usize, len: usize) -> TestResult {
            if len == 0 {
                TestResult::discard()
            } else {
                let i = LogIndex {
                    idx,
                    len,
                };
                TestResult::from_bool(i.physical() <= i.logical() && i.physical() < i.len)
            }
        }
    }
}
