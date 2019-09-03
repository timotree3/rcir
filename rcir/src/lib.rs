#![deny(warnings)]

pub fn find_winners<I, B, C>(ballots: I) -> Vec<C>
where
    I: IntoIterator<Item = B>,
    B: IntoIterator<Item = C>,
    C: Clone + std::hash::Hash + Eq,
{
    use std::collections::HashMap;

    // initialize candidate -> remaining voters map

    let mut voter_map: HashMap<C, Vec<B::IntoIter>> = HashMap::new();
    for ballot in ballots {
        let mut ballot = ballot.into_iter();
        if let Some(candidate) = ballot.next() {
            voter_map.entry(candidate).or_default().push(ballot);
        }
    }

    loop {
        let mut iter = voter_map
            .iter()
            .map(|(candidate, voters)| (candidate, voters.len()));

        // find best and worst candidates

        let (mut best_candidate, mut best_votecount) = match iter.next() {
            Some(pair) => pair,
            // no voters present, return empty list
            None => return Vec::new(),
        };
        let mut worst_candidates = vec![best_candidate];
        let mut worst_votecount = best_votecount;
        let mut total_votecount = best_votecount;
        for (candidate, votecount) in iter {
            if votecount < worst_votecount {
                worst_candidates.clear();
                worst_votecount = votecount;
            }
            if votecount <= worst_votecount {
                worst_candidates.push(candidate);
            } else if votecount > best_votecount {
                best_candidate = candidate;
                best_votecount = votecount;
            }
            total_votecount += votecount;
        }

        if best_votecount > total_votecount / 2 {
            // a candidate has the remaining majority, return a single winner
            return vec![best_candidate.clone()];
        }

        let worst_candidates = worst_candidates.into_iter().cloned().collect();
        if best_votecount == worst_votecount {
            // all the remaining candidates are tied, return a tie between them
            return worst_candidates;
        }

        // nobody has won yet, eliminate all the candidates tied for worst

        for candidate in &worst_candidates {
            let ballots_to_redistribute = voter_map.remove(candidate).unwrap();
            for mut ballot in ballots_to_redistribute {
                // find next remaining candidate on ballot, if any
                //
                // we can't use a for loop here because we reuse the iterator in the loop body
                while let Some(target) = ballot.next() {
                    if let Some(voters) = voter_map.get_mut(&target) {
                        voters.push(ballot);
                        break;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    #[allow(unused_imports)]
    use self::iter_assert::{deplete, neutral, undeplete};

    use super::find_winners;

    #[test]
    fn remaining_majority_wins() {
        let ballots = vec![
            undeplete(vec![0]),
            undeplete(vec![0]),
            deplete(vec![2, 1]),
            deplete(vec![3, 2]),
            deplete(vec![1, 3]),
        ];
        assert_eq!(&find_winners(ballots), &[0]);
    }

    #[test]
    fn first_round_absentees_never_win() {
        let ballots = vec![
            undeplete(vec![0]),
            undeplete(vec![0]),
            deplete(vec![1, 4]),
            deplete(vec![2, 4]),
            deplete(vec![3, 4]),
        ];
        assert_eq!(&find_winners(ballots), &[0]);
    }

    #[test]
    fn no_ballots_produces_no_winner() {
        let ballots: Vec<Vec<u64>> = vec![];
        assert_eq!(&find_winners(ballots), &[]);
    }

    #[test]
    fn all_empty_ballots_produces_no_winner() {
        let ballots = vec![deplete(vec![] as Vec<u64>), deplete(vec![])];
        assert_eq!(&find_winners(ballots), &[]);
    }

    #[test]
    fn initial_two_way_ties_work() {
        let ballots = vec![
            undeplete(vec![0]),
            undeplete(vec![0]),
            undeplete(vec![0]),
            undeplete(vec![1]),
            undeplete(vec![1]),
            undeplete(vec![1]),
        ];
        assert_eq!(
            find_winners(ballots).iter().collect::<HashSet<_>>(),
            [0, 1].iter().collect::<HashSet<_>>()
        );
    }

    #[test]
    fn initial_many_way_ties_work() {
        let ballots = vec![
            undeplete(vec![0]),
            undeplete(vec![0]),
            undeplete(vec![1]),
            undeplete(vec![1]),
            undeplete(vec![2]),
            undeplete(vec![2]),
            undeplete(vec![3]),
            undeplete(vec![3]),
            undeplete(vec![4]),
            undeplete(vec![4]),
        ];
        assert_eq!(
            find_winners(ballots).iter().collect::<HashSet<_>>(),
            [0, 1, 2, 3, 4].iter().collect::<HashSet<_>>()
        );
    }

    #[test]
    fn delayed_two_way_ties_work() {
        let ballots = vec![
            undeplete(vec![0]),
            undeplete(vec![0]),
            undeplete(vec![1]),
            undeplete(vec![1]),
            undeplete(vec![2, 1]),
            undeplete(vec![3, 0]),
        ];
        assert_eq!(
            find_winners(ballots).iter().collect::<HashSet<_>>(),
            [0, 1].iter().collect::<HashSet<_>>()
        );
    }

    #[test]
    fn delayed_many_way_ties_work() {
        let ballots = vec![
            undeplete(vec![0]),
            undeplete(vec![0]),
            undeplete(vec![0]),
            undeplete(vec![1]),
            undeplete(vec![1]),
            undeplete(vec![1]),
            undeplete(vec![2]),
            undeplete(vec![2]),
            undeplete(vec![2]),
            undeplete(vec![3]),
            undeplete(vec![3]),
            undeplete(vec![3]),
            undeplete(vec![4]),
            undeplete(vec![4]),
            undeplete(vec![5, 4]),
            deplete(vec![6, 5]),
        ];
        assert_eq!(
            find_winners(ballots).iter().collect::<HashSet<_>>(),
            [0, 1, 2, 3, 4].iter().collect::<HashSet<_>>()
        );
    }

    #[test]
    fn votes_count_behind_two_eliminations() {
        let ballots = vec![
            undeplete(vec![0]),
            undeplete(vec![0]),
            undeplete(vec![1, 2, 0]),
            undeplete(vec![4]),
            undeplete(vec![4]),
        ];
        assert_eq!(&find_winners(ballots), &[0]);
    }

    #[test]
    fn reference_candidates_work() {
        let a = "a";
        let b = "b";
        let c = "c";
        let d = "d";
        let ballots = vec![
            undeplete(vec![a]),
            undeplete(vec![a]),
            undeplete(vec![c, d, a]),
            undeplete(vec![b]),
            undeplete(vec![b]),
        ];
        assert_eq!(&find_winners(ballots), &[a]);
    }

    #[test]
    fn vecs_passed_by_reference_work() {
        let ballots = vec![
            undeplete(vec![0]),
            undeplete(vec![0]),
            undeplete(vec![1, 2, 0]),
            undeplete(vec![4]),
            undeplete(vec![4]),
        ];
        assert_eq!(&find_winners(ballots), &[0]);
    }

    // iterator testing helper functions to assert how far they will be polled
    mod iter_assert {
        use std::fmt::Debug;

        /// Wraps an Iterator to assert that it will be polled to completion.
        ///
        /// # Panics
        ///
        /// Panics if the returned wrapper is dropped before it is polled to produce a None.
        ///
        /// # Examples
        ///
        /// ```norun
        /// {
        ///     let mut iter = deplete(vec![3, 4, 5]);
        ///     println!("{:?}", iter.next()); // 3
        ///     println!("{:?}", iter.next()); // 4
        /// } // panic: iterator dropped before polled to completion
        ///
        /// println!("this line of code never runs :(");
        /// ```
        pub fn deplete<'a, V, I, A>(iter: V) -> IterAssert<I>
        where
            V: IntoIterator<Item = A, IntoIter = I> + 'a,
            I: Iterator<Item = A> + Debug,
        {
            IterAssert::<I> {
                inner: iter.into_iter(),
                kind: NeedsDeplete,
            }
        }

        /// Wraps an Iterator to assert that it will never be polled to completion.
        ///
        /// # Panics
        ///
        /// Panics if the returned wrapper is polled to produce a None.
        ///
        /// # Examples
        ///
        /// ```norun
        /// let mut iter = undeplete(vec![3, 4, 5]);
        /// println!("{:?}", iter.next()); // 3
        /// println!("{:?}", iter.next()); // 4
        /// println!("{:?}", iter.next()); // 5
        /// println!("{:?}", iter.next()); // panic: iterator polled to completion
        /// ```
        pub fn undeplete<'a, V, I, A>(iter: V) -> IterAssert<I>
        where
            V: IntoIterator<Item = A, IntoIter = I> + 'a,
            I: Iterator<Item = A> + Debug,
        {
            IterAssert::<I> {
                inner: iter.into_iter(),
                kind: Undeplete,
            }
        }

        /// Wraps an iterator, doing nothing in addition.
        /// To be used amongst other wrapped iterators to make them the same type.
        ///
        /// # Examples
        ///
        /// ```norun
        /// let iters: Vec<IterAssert<_>> = vec![
        ///     neutral(vec![1, 2, 3]),
        ///     deplete(vec![4, 5, 6]),
        ///     undeplete(vec![7, 8, 9]),
        /// ];
        /// ```
        #[allow(dead_code)]
        pub fn neutral<'a, V, I, A>(iter: V) -> IterAssert<I>
        where
            V: IntoIterator<Item = A, IntoIter = I> + 'a,
            I: Iterator<Item = A> + Debug,
        {
            IterAssert::<I> {
                inner: iter.into_iter(),
                kind: Satisfied,
            }
        }

        #[derive(Debug)]
        pub struct IterAssert<I> {
            inner: I,
            kind: IterAssertKind,
        }

        use self::IterAssertKind::*;

        #[derive(Debug)]
        enum IterAssertKind {
            Satisfied,
            NeedsDeplete,
            Undeplete,
        }

        impl<I: Iterator<Item = A>, A> Iterator for IterAssert<I> {
            type Item = A;

            fn next(&mut self) -> Option<A> {
                let next = self.inner.next();
                if next.is_none() {
                    match self.kind {
                        NeedsDeplete => self.kind = Satisfied,
                        Undeplete => panic!("iterator polled to completion"),
                        _ => {}
                    }
                }
                next
            }
        }

        impl<I> Drop for IterAssert<I> {
            fn drop(&mut self) {
                if let NeedsDeplete = self.kind {
                    if !std::thread::panicking() {
                        panic!("iterator dropped before polled to completion")
                    }
                }
            }
        }
    }
}
