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

    //<'a>(&'a mut self, arg: String) -> &'a mut Command

    pub fn of(&mut self, a: i32, b: i32) -> &mut IntervalSet {
        self.intervals.push(Interval::new(a, b));
        self
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

    pub fn is_empty(&self) -> bool {
        self.intervals.len() == 0
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

    pub fn add_set(&mut self, iset: &IntervalSet) -> Result<(), IntervalSetError> {
        if self.read_only {
            return Err(IntervalSetError::CantAlterReadOnly);
        } else {
            for m in iset.intervals.iter() {
                self.add(*m)?;
            }
            return Ok(());
        }
    }


    pub fn complement_range(&self, a:i32, b:i32) -> Option<IntervalSet> {
        return self.complement(&IntervalSet::new().of(a, b));
    }

    pub fn complement(&self, vocab:&IntervalSet) -> Option<IntervalSet> {
        if vocab.is_empty() {
            return None
        } else {
            return Some(vocab.subtract(self))
        }
    }

    pub fn subtract(&self, other: &IntervalSet) -> IntervalSet {
        subtract_intervalsets(self, other)
    }

    pub fn and(&self, other:&Option<&IntervalSet>) -> Option<IntervalSet> {
        if let Some(other) = other {
            let my_intervals = &self.intervals;
            let their_intervals = &other.intervals;
            let mut intersection:IntervalSet = IntervalSet::new();
            let my_size = my_intervals.len();
            let their_size = their_intervals.len();
            let mut i:i32 = 0;
            let mut j:i32 = 0;

            while (i as usize) < my_size && (j as usize) < their_size {
                let mine = &my_intervals[i as usize];
                let theirs = &their_intervals[j as usize];
                if mine.starts_before_disjoint(&theirs) {
                    i += 1;
                } else if theirs.starts_before_disjoint(&mine) {
                    j += 1;
                } else if mine.properly_contains(&theirs) {
                    // TODO: deal with this result
                    let _ = intersection.add(mine.intersection(theirs));
                    j = j+1;
                } else if theirs.properly_contains(&mine) {
                    let _ = intersection.add(mine.intersection(theirs));
                    j = j+1;
                } else if !mine.disjoint(theirs) {
                    let _ = intersection.add(mine.intersection(theirs));
                    if mine.starts_after_non_disjoint(theirs) {
                        j = j + 1;
                    } else if theirs.starts_after_non_disjoint(mine) {
                        i = i + 1;
                    }
                }
            }
            return Some(intersection);
        } else {
            return None
        }
    }

    pub fn contains(&self, el:i32) -> bool {
        let n = self.intervals.len();
		let mut l = 0;
		let mut r = n - 1;
		// Binary search for the element in the (sorted,
		// disjoint) array of intervals.
		while l <= r {
			let m = (l + r) / 2;
			let ival:Interval = self.intervals[m];
			let a = ival.a;
			let b = ival.b;
			if b < el {
				l = m + 1;
			} else if  a>el  {
				r = m - 1;
			} else { // el >= a && el <= b
				return true;
			}
		}
		return false;
    }

    pub fn get_max_element(&self) -> Option<i32> {
        self.intervals.last().map(|l:&Interval| l.b)
    }

    pub fn get_min_element(&self) -> Option<i32> {
        self.intervals.first().map(|l:&Interval| l.a)
    }

    pub fn size(&self) -> i32 {
        let mut n = 0;
		let num_intervals = self.intervals.len();
		if num_intervals==1 {
			let first_interval:Interval = self.intervals[0];
			return first_interval.b-first_interval.a+1;
		}
        for i in 0..num_intervals {
            let ival = self.intervals[i];
            n += ival.b - ival.a + 1;
        }
		return n;
    }

    pub fn to_integer_list(&self) -> Vec<i32> {
        let mut values = Vec::new();
        let n = self.intervals.len();
        for i in 0..n {
            let ival = self.intervals[i];
            for v in ival.a..=ival.b {
                values.push(v);
            }
        }
        return values;
    }

    fn element_name(vocabulary:String) -> String {
        // TODO
        // TODO: create vocabulary
    }

    pub fn remove(&mut self, el:i32) {
        // TODO
    }

    pub fn set_read_only(&mut self, bool v) {
        if self.read_only && !v {
            // TODO
            panic!("Can't alter readonly IntervalSet")
        }
        self.read_only = v;
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

pub fn subtract_intervalsets(left: &IntervalSet, right: &IntervalSet) -> IntervalSet {
    if left.is_empty() {
        return IntervalSet::new()
    }

    let mut result = IntervalSet::new_from_intervals(left.intervals.clone());
    if right.is_empty() {
        // right set has no elements; just return the copy of the current set
        return result
    } 

    let mut result_i:i32 = 0;
    let mut right_i:i32 = 0;
    while (result_i as usize) < result.intervals.len() && (right_i as usize) < right.intervals.len() {
        let result_interval = result.intervals[result_i as usize];
        let right_interval = right.intervals[right_i as usize];

        // operation: (resultInterval - rightInterval) and update indexes

        if right_interval.b < result_interval.a {
            right_i += 1;
            continue;
        }

        if right_interval.a > result_interval.b {
            result_i += 1;
            continue;
        }

        let mut before_current:Option<Interval> = None;
        let mut after_current:Option<Interval> = None;
        
        if right_interval.a > result_interval.a {
            before_current = Some(Interval::new(result_interval.a, right_interval.a - 1));
        }

        if right_interval.b < result_interval.b {
            after_current = Some(Interval::new(right_interval.b + 1, result_interval.b));
        }


        if before_current.is_some() {
            if after_current.is_some() {
                // split the current interval into two
                result.intervals[result_i as usize] = before_current.unwrap();
                result.intervals.insert(result_i as usize +1 , after_current.unwrap());
                result_i += 1;
                continue;
            }
            else {
                // replace the current interval
                result.intervals[result_i as usize] = before_current.unwrap();
                result_i += 1;
                continue;
            }
        }
        else {
            if after_current.is_some() {
                // replace the current interval
                result.intervals[result_i as usize] = after_current.unwrap();
                right_i += 1;
                continue;
            }
            else {
                // remove the current interval (thus no need to increment resultI)
                result.intervals.remove(result_i as usize);
                continue;
            }
        }
    }

    // If rightI reached right.intervals.size(), no more intervals to subtract from result.
    // If resultI reached result.intervals.size(), we would be subtracting from an empty set.
    // Either way, we are done.
    return result
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
