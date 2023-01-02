use std::fmt;
use std::ops::{Add, Div};

pub struct Statistic<T> {
    min: Option<T>,
    max: Option<T>,
    sum: Option<T>,
    cnt: u32,
}
impl<T: Ord + Copy + Add<T, Output = T>> Statistic<T> {
    pub fn new() -> Self {
        Self {
            cnt: 0,
            sum: None,
            min: None,
            max: None,
        }
    }
    pub fn push(&mut self, value: T) {
        self.cnt += 1;
        self.sum = match self.sum {
            Some(sum) => Some(sum + value),
            None => Some(value),
        };
        self.min = match self.min {
            Some(min) => Some(std::cmp::min(min, value)),
            None => Some(value),
        };
        self.max = match self.max {
            Some(max) => Some(std::cmp::max(max, value)),
            None => Some(value),
        };
    }
}

impl<T: fmt::Debug + Copy + Div<u32, Output = T> + Clone> fmt::Debug for Statistic<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Stats")
            .field("cnt", &self.cnt)
            .field("min", &self.min.unwrap())
            .field("avg", &(self.sum.unwrap() / self.cnt))
            .field("max", &self.max.unwrap())
            .finish()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_duration() {
        use std::time::Duration;
        let mut stat = Statistic::new();
        stat.push(Duration::from_secs(1));
        stat.push(Duration::from_secs(2));
        stat.push(Duration::from_secs(3));
        assert_eq!(
            format!("{stat:?}"),
            "Stats { cnt: 3, min: 1s, avg: 2s, max: 3s }"
        );
        stat.push(Duration::from_secs(0));
        assert_eq!(
            format!("{stat:?}"),
            "Stats { cnt: 4, min: 0ns, avg: 1.5s, max: 3s }"
        );
        stat.push(Duration::from_secs(0));
        assert_eq!(
            format!("{stat:?}"),
            "Stats { cnt: 5, min: 0ns, avg: 1.2s, max: 3s }"
        );
    }
    #[test]
    fn test_u32() {
        let mut stat = Statistic::new();
        stat.push(1);
        stat.push(2);
        stat.push(3);
        assert_eq!(
            format!("{stat:?}"),
            "Stats { cnt: 3, min: 1, avg: 2, max: 3 }"
        );
        stat.push(0);
        assert_eq!(
            format!("{stat:?}"),
            "Stats { cnt: 4, min: 0, avg: 1, max: 3 }"
        );
        stat.push(0);
        assert_eq!(
            format!("{stat:?}"),
            "Stats { cnt: 5, min: 0, avg: 1, max: 3 }"
        );
    }
}
