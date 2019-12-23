use std::cmp::max;
use std::cmp::min;
use std::fmt;

pub use crate::antlr4::runtime::token::TokenType;

/** An immutable inclusive interval a..b */
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Interval {
    pub a: i32,
    pub b: i32, // stop is not included
}

pub enum IntervalSetError {
    CantAlterReadOnly,
}

impl Interval {
    pub fn new(a: i32, b: i32) -> Interval {
        Interval { a: a, b: b }
    }

    pub fn contains(&self, item: i32) -> bool {
        item >= self.a && item <= self.b
    }

    pub fn length(&self) -> i32 {
        if self.b < self.a {
            return 0;
        } else {
            return self.b - self.a;
        }
    }

    /** Does self start completely before other? Disjoint */
    pub fn starts_before_disjoint(&self, other: &Interval) -> bool {
        return self.a < other.a && self.b < other.a;
    }

    /** Does self start at or before other? Nondisjoint */
    pub fn starts_before_non_disjoint(&self, other: &Interval) -> bool {
        return self.a <= other.a && self.b >= other.a;
    }

    /** Does self.a start after other.b? May or may not be disjoint */
    pub fn starts_after(&self, other: &Interval) -> bool {
        return self.a > other.a;
    }

    /** Does self start completely after other? Disjoint */
    pub fn starts_after_disjoint(&self, other: &Interval) -> bool {
        return self.a > other.b;
    }

    /** Does self start after other? NonDisjoint */
    pub fn starts_after_non_disjoint(&self, other: &Interval) -> bool {
        return self.a > other.a && self.a <= other.b; // self.b>=other.b implied
    }

    /** Are both ranges disjoint? I.e., no overlap? */
    pub fn disjoint(&self, other: &Interval) -> bool {
        return self.starts_before_disjoint(other) || self.starts_after_disjoint(other);
    }

    /** Are two intervals adjacent such as 0..41 and 42..42? */
    pub fn adjacent(&self, other: &Interval) -> bool {
        return self.a == other.b + 1 || self.b == other.a - 1;
    }

    pub fn properly_contains(&self, other: &Interval) -> bool {
        return other.a >= self.a && other.b <= self.b;
    }

    /** Return the interval computed from combining self and other */
    pub fn union(&self, other: &Interval) -> Interval {
        return Interval::new(min(self.a, other.a), max(self.b, other.b));
    }

    /** Return the interval in common between self and o */
    pub fn intersection(&self, other: &Interval) -> Interval {
        return Interval::new(max(self.a, other.a), min(self.b, other.b));
    }

    /** Return the interval with elements from this not in other;
     *  other must not be totally enclosed (properly contained)
     *  within this, which would result in two disjoint intervals
     *  instead of the single one returned by this method.
     */
    pub fn difference_not_properly_contained(&self, other: &Interval) -> Option<Interval> {
        // other.a to left of this.a (or same)
        if other.starts_before_non_disjoint(self) {
            return Some(Interval::new(max(self.a, other.b + 1), self.b));
        } else if other.starts_after_non_disjoint(self) {
            // other.a to right of this.a
            return Some(Interval::new(self.a, other.a - 1));
        } else {
            return None;
        }
    }
}

impl fmt::Display for Interval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.a, self.b)
    }
}

pub struct IntervalSet {
    // in the Golang impl, the intervals collection is initialized upon add_interval
    // not sure if it's worth
    intervals: Vec<Interval>,
    read_only: bool,
}

impl IntervalSet {
    pub fn new() -> IntervalSet {
        IntervalSet {
            intervals: Vec::new(),
            read_only: false,
        }
    }

    pub fn new_from_intervals(ivs: Vec<Interval>) -> IntervalSet {
        IntervalSet {
            intervals: ivs,
            read_only: false,
        }
    }

    pub fn of(&mut self, a: i32, b: i32) {
        self.intervals.push(Interval::new(a, b));
    }

    // TODO: better name?
    pub fn of_same(&mut self, a: i32) {
        self.intervals.push(Interval::new(a, a));
    }

    pub fn clear(&mut self) -> Result<(), &str> {
        if self.read_only {
            return Err("can't alter readonly IntervalSet");
        } else {
            self.intervals.clear();
            return Ok(());
        }
    }

    pub fn first(&self) -> Result<i32, TokenType> {
        if self.intervals.len() == 0 {
            return Err(TokenType::InvalidType);
        }
        return Ok(self.intervals[0].a);
    }

    fn add(&mut self, addition: Interval) -> Result<(), IntervalSetError> {
        if self.read_only {
            return Err(IntervalSetError::CantAlterReadOnly);
        } else {
            for index in 0..self.intervals.len() {
                let r = self.intervals[index];
                if addition == r {
                    return Ok(());
                }

                if addition.adjacent(&r) || !addition.disjoint(&r) {
                    // next to each other, make a single larger interval
                    let bigger: Interval = addition.union(&r);
                    self.intervals[index] = bigger;

                    // make sure we didn't just create an interval that
                    // should be merged with next interval in list
                    let mut i = index;
                    while i < self.intervals.len() - 1 {
                        i += 1;
                        let next = self.intervals[i];
                        if !bigger.adjacent(&next) && bigger.disjoint(&next) {
                            break;
                        }
                        let even_bigger = bigger.union(&next);
                        self.intervals.remove(i);
                        i -= 1;
                        self.intervals[i] = even_bigger;
                    }
                    return Ok(());
                }
                if addition.starts_before_disjoint(&r) {
                    // insert before r
                    self.intervals.insert(index, addition);
                    return Ok(());
                }
            }
            return Ok(());
        }
    }

    pub fn add_all(&mut self, iset: IntervalSet) -> Result<(), IntervalSetError> {
        if self.read_only {
            return Err(IntervalSetError::CantAlterReadOnly);
        } else {
            for m in iset.intervals.iter() {
                self.add(*m)?;
            }
            return Ok(());
        }
    }
}

impl fmt::Display for IntervalSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let l = self
            .intervals
            .iter()
            .map(|&x| x.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        write!(f, "[{}]", l)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interval() {
        let x = Interval::new(1, 5);
        assert_eq!("1..5".to_string(), x.to_string());

        for i in 1..5 {
            assert!(x.contains(i));
        }
        assert!(!x.contains(6));
    }

    #[test]
    fn test_set_stuff() {
        let x = Interval::new(1, 5);
        let y = Interval::new(6, 10);
        let z = Interval::new(2, 6);

        assert!(x.starts_before_disjoint(&y));
        assert!(x.starts_before_non_disjoint(&z));
        assert!(!y.starts_before_non_disjoint(&z));
        // TODO, although they were copied from the Java source
    }

    #[test]
    fn test_interval_set_merge() {
        let vs = vec![Interval::new(1, 4), Interval::new(7, 8)];
        let mut iset = IntervalSet::new_from_intervals(vs);
        let z = Interval::new(2, 6);
        assert!(iset.add(z).is_ok());
        assert_eq!(iset.intervals.len(), 1);
        assert_eq!(iset.intervals[0], Interval::new(1, 8));
    }

    #[test]
    fn test_interval_set_extended_merge() {
        let vs = vec![
            Interval::new(1, 4),
            Interval::new(7, 8),
            Interval::new(10, 12),
        ];
        let mut iset = IntervalSet::new_from_intervals(vs);
        let z = Interval::new(2, 13);
        assert!(iset.add(z).is_ok());
        assert_eq!(iset.intervals.len(), 1);
        assert_eq!(iset.intervals[0], Interval::new(1, 13));
    }

    #[test]
    fn test_interval_set_middle() {
        let vs = vec![Interval::new(1, 4), Interval::new(10, 12)];
        let mut iset = IntervalSet::new_from_intervals(vs);
        let z = Interval::new(6, 8);
        assert!(iset.add(z).is_ok());
        assert_eq!(iset.intervals.len(), 3);
        assert_eq!(iset.intervals[0], Interval::new(1, 4));
        assert_eq!(iset.intervals[1], Interval::new(6, 8));
        assert_eq!(iset.intervals[2], Interval::new(10, 12));
    }

    #[test]
    fn test_interval_set_first() {
        let vs = vec![Interval::new(4, 5), Interval::new(10, 12)];
        let mut iset = IntervalSet::new_from_intervals(vs);
        let z = Interval::new(1, 2);
        assert!(iset.add(z).is_ok());
        assert_eq!(iset.intervals.len(), 3);
        assert_eq!(iset.intervals[0], Interval::new(1, 2));
        assert_eq!(iset.intervals[1], Interval::new(4, 5));
        assert_eq!(iset.intervals[2], Interval::new(10, 12));
    }
}
