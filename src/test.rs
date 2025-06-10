use std::{cmp::Ordering, collections::BTreeSet};

struct Solution;
impl Solution {
    pub fn longest_valid_parentheses(s: String) -> i32 {
        let prefix = std::iter::once(0).chain(
        s
        .chars()
        .into_iter()
        .map(|c| match c {
            '(' => 1,
            ')' => -1,
            _ => unreachable!()
        })
        .scan(0, |acc, x| {
            *acc = *acc + x;
            Some(*acc)
        }))
        .collect::<Vec<_>>();

        let mut sorted_with_index = prefix
        .into_iter()
        .enumerate()
        .collect::<Vec<_>>();
        sorted_with_index.sort_by(|(a,b),(c,d)| 
            match b.cmp(d) {
                Ordering::Less => Ordering::Less,
                Ordering::Equal => a.cmp(c),
                Ordering::Greater => Ordering::Greater 
            }
        );

        //println!("{:?}", sorted_with_index);
        let mut state = (0, sorted_with_index[0].1);
        let range_with_diff = sorted_with_index.iter()
        .cloned()
        .enumerate()
        .skip(1)
        .chain(std::iter::once((sorted_with_index.len(), (0, 0))))
        .map(|(id, (_, val))| {
            let (last_id, last_val) = state;
            //println!("last_id:{:?}, id:{:?} val:{:?}",last_id, id, val);
            if id == sorted_with_index.len() {
                return Some((last_id.clone(), id - 1));
            }
            
            if val != last_val {
                
                let res = (last_id.clone(), id - 1);
                state.0 = id;
                state.1 = val;
                Some(res)
            } else {
                None
            }
        })
        .flatten()
        .collect::<Vec<(usize, usize)>>();
        //println!("{:?}", range_with_diff);

        let mut s = BTreeSet::new();
        range_with_diff.into_iter()
        .map(| (l,r)| {
            
            let mut last_id : Option<usize> = None;
            let mut max_length : i32 = 0;
            sorted_with_index[l..=r].iter().for_each(
                |x| {
                    let (id, _) =  x;
                    match (last_id, s.range(..id).next_back()) {
                        (None, _) => {last_id = Some(*id);},
                        (Some(l_id), None) => {
                            max_length = max_length.max((id - l_id) as i32);
                        },
                        (Some(l_id), Some(&pred)) => {
                            if l_id <= pred {
                                last_id = Some(*id);
                            } else {
                                max_length = max_length.max((id - l_id) as i32);
                            }
                        }
                    }
                }
            );
            
            sorted_with_index[l..=r].iter().for_each(
                |(id,_)| {
                    s.insert(*id);
                }
            );
            Some(max_length)
        }
        ).flatten()
        .max()
        .unwrap_or(0)
    }
}