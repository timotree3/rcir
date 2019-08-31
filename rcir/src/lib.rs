#![deny(warnings)]

type Ballot = std::vec::IntoIter<u64>;

pub fn find_winners(ballots: Vec<Vec<u64>>) -> Vec<u64> {
    use std::collections::HashMap;

    // initialize candidate -> remaining voters map

    let mut voter_map: HashMap<u64, Vec<Ballot>> = HashMap::new();
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
        let mut worst_candidates = vec![*best_candidate];
        let mut worst_votecount = best_votecount;
        let mut total_votecount = best_votecount;
        for (candidate, votecount) in iter {
            if votecount < worst_votecount {
                worst_candidates.clear();
                worst_votecount = votecount;
            }
            if votecount <= worst_votecount {
                worst_candidates.push(*candidate);
            } else if votecount > best_votecount {
                best_candidate = candidate;
                best_votecount = votecount;
            }
            total_votecount += votecount;
        }

        if best_votecount > total_votecount / 2 {
            // a candidate has the remaining majority, return a single winner
            return vec![*best_candidate];
        }
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
    use super::find_winners;

    #[test]
    fn remaining_majority_wins() {
        let ballots = vec![vec![0, 1], vec![0, 3], vec![2, 1], vec![3, 2], vec![1, 3]];
        assert_eq!(&find_winners(ballots), &[0]);
    }

    #[test]
    fn first_round_absentees_never_win() {
        let ballots = vec![vec![0, 4], vec![0, 4], vec![1, 4], vec![2, 4], vec![3, 4]];
        assert_eq!(&find_winners(ballots), &[0]);
    }

    #[test]
    fn no_ballots_produces_no_winner() {
        let ballots = vec![];
        assert_eq!(&find_winners(ballots), &[]);
    }

    #[test]
    fn all_empty_ballots_produces_no_winner() {
        let ballots = vec![vec![], vec![]];
        assert_eq!(&find_winners(ballots), &[]);
    }

    #[test]
    fn initial_two_way_ties_work() {
        let ballots = vec![
            vec![0, 1, 2],
            vec![0, 2, 1],
            vec![0, 3, 3],
            vec![1, 4, 5],
            vec![1, 5, 0],
            vec![1, 0, 4],
        ];
        let mut result = find_winners(ballots);
        result.sort();
        assert_eq!(&result, &[0, 1]);
    }

    #[test]
    fn initial_many_way_ties_work() {
        let ballots = vec![
            vec![0, 1, 2],
            vec![0, 1, 2],
            vec![1, 0, 2],
            vec![1, 0, 2],
            vec![2, 0, 1],
            vec![2, 0, 1],
            vec![3, 0, 2],
            vec![3, 0, 2],
            vec![4, 0, 2],
            vec![4, 0, 2],
        ];
        let mut result = find_winners(ballots);
        result.sort();
        assert_eq!(&result, &[0, 1, 2, 3, 4]);
    }

    #[test]
    fn delayed_two_way_ties_work() {
        let ballots = vec![
            vec![0, 1],
            vec![0, 2],
            vec![1, 3],
            vec![1, 4],
            vec![2, 1],
            vec![3, 0],
        ];
        let mut result = find_winners(ballots);
        result.sort();
        assert_eq!(&result, &[0, 1]);
    }

    #[test]
    fn delayed_many_way_ties_work() {
        let ballots = vec![
            vec![0, 1],
            vec![0, 2],
            vec![0, 3],
            vec![1, 4],
            vec![1, 5],
            vec![1, 6],
            vec![2, 7],
            vec![2, 8],
            vec![2, 9],
            vec![3, 0],
            vec![3, 1],
            vec![3, 2],
            vec![4, 3],
            vec![5, 4],
            vec![4, 5],
        ];
        let mut result = find_winners(ballots);
        result.sort();
        assert_eq!(&result, &[0, 1, 2, 3, 4]);
    }
}
