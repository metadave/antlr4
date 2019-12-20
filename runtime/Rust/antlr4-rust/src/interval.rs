use std::fmt;
use std::cmp::min;
use std::cmp::max;

pub use crate::token::TokenType;

#[derive(Debug, PartialEq)]
pub struct Interval {
    pub a: i32,
    pub b: i32, // stop is not included
}

impl Interval {
    pub fn new(a: i32, b: i32) -> Interval {
        Interval { a: a, b: b }
    }

    pub fn contains(&self, item: i32) -> bool {
        item >= self.a && item < self.b
    }

    pub fn string(&self) -> String {
        if self.a == self.b - 1 {
            return self.a.to_string();
        }
        return self.to_string();
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
	pub fn difference_not_properly_contained(&self, other:&Interval) -> Option<Interval> {
        // other.a to left of this.a (or same)
		if other.starts_before_non_disjoint(self) {
			return Some(Interval::new(max(self.a, other.b + 1),
							   self.b));
		} else if other.starts_after_non_disjoint(self) {
            // other.a to right of this.a
            return Some(Interval::new(self.a, other.a - 1));
		} else {
            return None
        }
	}
}

impl fmt::Display for Interval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.a == self.b - 1 {
            write!(f, "{}", self.a)
        } else {
            write!(f, "{}..{}", self.a, self.b - 1)
        }
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

    pub fn first(&self) -> Result<i32, TokenType> {
        if self.intervals.len() == 0 {
            return Err(TokenType::InvalidType);
        }
        return Ok(self.intervals[0].a);
    }

    fn add(&self, addition: Interval) -> Result<(), &str> {
        if self.read_only {
            return Err("Can't alter readonly IntervalSet");
        } else {
            for r in &self.intervals {
                println!("{:?}", r);
            }
            return Ok(());
        }
    }

    // pub fn add_one(&self, v:i32) {
    //     self.
    // }
    // func (i *IntervalSet) addOne(v int) {
    //     i.addInterval(NewInterval(v, v+1))
    // }
}
