use serde::Deserialize;
use std::{cell::RefCell, collections::HashMap, fmt::Debug};

#[derive(Clone, PartialEq, Eq)]
struct Pat {
    s: Vec<u8>,
    rle: bool,
}

impl Debug for Pat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", std::str::from_utf8(&self.s).unwrap())?;
        if self.rle {
            write!(f, "rle")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Sol {
    res: String,
    pats: Vec<Pat>,
    bytes: isize,
}

impl std::cmp::PartialOrd for Sol {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.bytes.partial_cmp(&other.bytes)
    }
}

impl std::cmp::Ord for Sol {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.bytes.cmp(&other.bytes)
    }
}

#[derive(Deserialize)]
struct Puzzle {
    puzzle: String,
}

const FAKE: u8 = 'Ф' as u8;

fn count_cost(count: usize) -> isize {
    match count {
        0 => 0,
        1..=2 => 1,
        3..=4 => 2,
        5..=8 => 3,
        _ => unreachable!(),
    }
}

fn solve_brute(start_pat_count: usize, pat_count: usize, max_pat_count: usize, s: &[u8]) -> Sol {
    thread_local! {
        static MEMO: RefCell<HashMap<(usize,usize, usize, Vec<u8>), Sol>> = RefCell::new(HashMap::new());
    }
    MEMO.with(|memo| {
        let key = (start_pat_count, pat_count, max_pat_count, s.to_vec());
        if let Some(result) = memo.borrow().get(&key) {
            return result.clone();
        }
        // println!("Now solving {:?}", std::str::from_utf8(s).unwrap());
        let mut result = Sol {
            res: std::str::from_utf8(s).unwrap().to_owned(),
            pats: vec![],
            bytes: s.iter().filter(|&&c| c != FAKE).count() as isize + count_cost(pat_count)
                - count_cost(start_pat_count),
        };
        if pat_count == max_pat_count {
            return result;
        }
        for i in 0..s.len() {
            let tail = &s[i..];
            for (j, c) in tail.iter().copied().enumerate() {
                if c == FAKE {
                    break;
                }
                let sub = &tail[..=j];
                if sub.len() > 12 {
                    break;
                }

                let mut new_s = Vec::new();

                let mut i = 0;
                let mut count = 0;
                let mut non_rle_bytes = 0;
                let mut rle_bytes = 0;
                let mut prev_fake = false;
                let mut original = 0;
                while i < s.len() {
                    if s[i..].starts_with(sub) {
                        if !prev_fake {
                            new_s.push(FAKE);
                            prev_fake = true;
                        }
                        count += 1;
                        non_rle_bytes += 1;
                        i += sub.len();
                        original += sub.len();
                    } else {
                        if count != 0 {
                            rle_bytes += 3;
                        }
                        count = 0;
                        new_s.push(s[i]);
                        prev_fake = false;
                        i += 1;
                    }
                }
                if count != 0 {
                    rle_bytes += 3;
                }
                if (original as isize)
                    < sub.len() as isize + std::cmp::min(rle_bytes, non_rle_bytes)
                {
                    continue;
                }

                let sol = solve_brute(start_pat_count, pat_count + 1, max_pat_count, &new_s);
                for rle in [false, true] {
                    let mut sol = sol.clone();
                    sol.pats.push(Pat {
                        s: sub.to_vec(),
                        rle,
                    });
                    sol.bytes += sub.len() as isize;
                    if rle {
                        sol.bytes += rle_bytes;
                    } else {
                        sol.bytes += non_rle_bytes;
                    }
                    result = std::cmp::min(result, sol);
                }
            }
        }
        memo.borrow_mut().insert(key, result.clone());
        result
    })
}

fn solve(s: &[u8], max_at_once: usize) -> Sol {
    let mut result = Sol {
        res: std::str::from_utf8(s).unwrap().to_owned(),
        pats: vec![],
        bytes: s.len() as isize,
    };
    while result.pats.len() < 8 {
        println!("batching from {}", result.pats.len());
        let mut batch = solve_brute(
            result.pats.len(),
            result.pats.len(),
            std::cmp::min(8, result.pats.len() + max_at_once),
            result.res.as_bytes(),
        );
        if batch.pats.is_empty() {
            break;
        }
        batch.pats.reverse();
        result.res = batch.res;
        result.pats.extend(batch.pats);
    }
    result
}

fn main() {
    let n: usize = std::env::args().nth(1).unwrap().parse().unwrap();
    let s: Vec<Puzzle> =
        serde_json::from_reader(std::fs::File::open("badcop.json").unwrap()).unwrap();
    let s = &s[n - 1].puzzle;
    println!("solving {:#?}", s);
    println!(
        "{:#?}",
        solve(
            s.as_bytes(),
            std::env::args().nth(2).unwrap().parse().unwrap()
        )
    );
}
