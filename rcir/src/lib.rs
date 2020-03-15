#![deny(warnings)]

use std::{collections::HashMap, hash::Hash};

/// Runs an RCIR election, resolving ties by acting equally upon the tied candidates
///
/// Takes an iterator of ballots where each ballot ranks candidates,
/// starting with first choice and ending with last. If some candidates tie, returns
/// a vector containing all of them. If all ballots are empty, returns an empty vector.
///
/// It is a logic error if `ballots` produces more than `usize::MAX` elements.
///
/// # Time Complexity
///
/// The worst-case scenario for this algorithm is when the candidates are close
/// enough that the voting makes it to a round of two.
///
/// In that case, the time complexitiy for this algorithm is O(V+(C^2))
/// where V is the number of observable votes
/// and C is the number of first round candidates
///
/// # Examples
///
/// Basic usage
///
/// ```rust
/// # use rcir::find_winners;
/// // this election contains six votes
/// // four observable votes
/// // and two first round candidates
///
/// let ballots = vec![
///     vec![0, 3],
///     vec![0, 3],
/// // the 3s above are unobservable because they are ranked after
/// // a vote for the winner
///
///     vec![1, 2],
/// ];
///
/// assert_eq!(&find_winners(ballots), &[0]);
///
/// // this election contains eight votes
/// // eight observable votes
/// // and four first round candidates
///
/// let ballots = vec![
///     vec![0],
///     vec![0],
///
/// // these three candidates get eliminated because they are tied with each other
///     vec![2, 1],
///     vec![3, 2],
///     vec![1, 3],
/// ];
///
/// assert_eq!(&find_winners(ballots), &[0]);
/// ```
pub fn find_winners<I, B, C>(ballots: I) -> Vec<C>
where
    I: IntoIterator<Item = B>,
    B: IntoIterator<Item = C>,
    C: std::hash::Hash + Eq,
{
    // initialize candidate -> remaining voters map

    let mut voter_map: HashMap<C, Vec<B::IntoIter>> = HashMap::new();
    for ballot in ballots {
        let mut ballot = ballot.into_iter();
        if let Some(candidate) = ballot.next() {
            voter_map.entry(candidate).or_default().push(ballot);
        }
    }

    loop {
        let votecounts = voter_map.values().map(|voters| voters.len());

        let best_votecount = votecounts.clone().max();
        let worst_votecount = votecounts.clone().min();

        let (best_votecount, worst_votecount) = match (best_votecount, worst_votecount) {
            (Some(b), Some(w)) => (b, w),
            // no voters present, return empty list
            (None, None) => return Vec::new(),
            _ => unreachable!("if Iterator::min() returns Some(v), so should Iterator::max()"),
        };

        // here we assume overflow doesn't occur, see the `fn` level docs
        let total_votecount: usize = votecounts.sum();

        if best_votecount > total_votecount / 2 {
            // a candidate has the remaining majority, return a single winner
            let mut winners = voter_map
                .into_iter()
                .filter(|(_candidate, voters)| voters.len() == best_votecount);
            let (winner, _votecount) = winners
                .next()
                .expect("candidate with best votecount to remain in voter map");
            debug_assert!(winners.next().is_none());
            return vec![winner];
        }

        if best_votecount == worst_votecount {
            // all the remaining candidates are tied, return a tie between them
            let winners = voter_map
                .into_iter()
                .map(|(candidate, _voters)| candidate)
                .collect();
            return winners;
        }

        // nobody has won yet, eliminate all the candidates tied for worst

        let worst_candidates = hash_map_drain_filter(&mut voter_map, |(_candidate, voters)| {
            voters.len() == worst_votecount
        });

        for (_candidate, voters) in worst_candidates {
            for mut ballot in voters {
                // find next remaining candidate on ballot, if any
                //
                // we can't use a for loop here because we reuse the iterator
                // in the loop body
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

// FIXME: replace with HashMap::drain_filter once it exists/is stable
fn hash_map_drain_filter<K: Hash + Eq, V, F: FnMut(&mut (K, V)) -> bool>(
    map: &mut HashMap<K, V>,
    mut should_drain: F,
) -> Vec<(K, V)> {
    let mut vec = Vec::new();
    let owned_map = std::mem::replace(map, HashMap::default());
    let owned_map = owned_map
        .into_iter()
        .filter_map(|mut entry| {
            if should_drain(&mut entry) {
                vec.push(entry);
                None
            } else {
                Some(entry)
            }
        })
        .collect();
    std::mem::replace(map, owned_map);
    vec
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

    // In some initial versions of the "majority found" early-exit logic,
    // it counted exactly half of the votes as a uniquely winning majority.
    //
    // This tests that we don't exit early on exactly half votes and
    // instead wait until later to see if another candidate gains half votes
    // as well such that we arrive at a two way tie.
    #[test]
    fn initally_half_later_two_way_tie() {
        let ballots = vec![
            undeplete(vec![0]),
            undeplete(vec![0]),
            undeplete(vec![0]),
            undeplete(vec![1]),
            undeplete(vec![1]),
            undeplete(vec![2, 1]),
        ];
        assert_eq!(
            find_winners(ballots).iter().collect::<HashSet<_>>(),
            [0, 1].iter().collect::<HashSet<_>>()
        );
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
