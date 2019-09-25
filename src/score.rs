extern crate fnv;

use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::hash::{Hash, Hasher};

use fnv::{FnvHashMap, FnvHashSet};

const NGRAM_LEN : usize = 4;
const NGRAM2_LEN : usize = 2;

pub struct NgramScore4 {
    ngram_map : FnvHashMap<NgramText4, i64>,
    pub total : f64
}

pub struct NgramScore2 {
    ngram_map : FnvHashMap<NgramText2, i64>,
    pub total : f64
}

struct NgramText4 {
    ngram : [char; NGRAM_LEN]
}

impl From<&str> for NgramText4 {
    fn from(s : &str) -> Self {
        assert_eq!(s.len(), NGRAM_LEN);

        let mut it = s.chars();
        NgramText4 { ngram: [
            it.next().unwrap(),
            it.next().unwrap(),
            it.next().unwrap(),
            it.next().unwrap(),
        ]}
    }
}

impl PartialEq for NgramText4 {
    fn eq(&self, other: &Self) -> bool {
        self.ngram[0] == other.ngram[0] &&
        self.ngram[1] == other.ngram[1] &&
        self.ngram[2] == other.ngram[2] &&
        self.ngram[3] == other.ngram[3]
    }
}

impl Eq for NgramText4 { }

impl Hash for NgramText4 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_i8(self.ngram[0] as i8);
        state.write_i8(self.ngram[1] as i8);
        state.write_i8(self.ngram[2] as i8);
        state.write_i8(self.ngram[3] as i8);
    }
}

struct NgramText2 {
    ngram : [char; NGRAM2_LEN]
}

impl From<&str> for NgramText2 {
    fn from(s : &str) -> Self {
        assert_eq!(s.len(), NGRAM2_LEN);

        let mut it = s.chars();

        NgramText2 { ngram: [
            it.next().unwrap(),
            it.next().unwrap(),
        ]}
    }
}

impl PartialEq for NgramText2 {
    fn eq(&self, other: &Self) -> bool {
        self.ngram[0] == other.ngram[0] &&
        self.ngram[1] == other.ngram[1]
    }
}

impl Eq for NgramText2 { }

impl Hash for NgramText2 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_i8(self.ngram[0] as i8);
        state.write_i8(self.ngram[1] as i8);
    }
}


impl NgramScore2 {
    pub fn create(filename: &str) -> Self {
        let file = File::open(filename)
            .expect(&format!("Cannot open {}", filename));

        let file = BufReader::new(file);

        let ngram_map : FnvHashMap<_, _> = file.lines()
            .map(|l| l.unwrap())
            .map(|l| {
                let mut it = l.split_ascii_whitespace();
                let qgram = it.next().unwrap().to_uppercase();
                let n = it.next().unwrap().parse::<i64>().unwrap();

                qgram.chars().for_each(|c| {
                    assert!('A' <= c && c <= 'Z');
                });
                assert_eq!(it.next(), None);

                (NgramText2::from(qgram.as_str()), n)
            })
            .collect();
        
        let total : f64 = ngram_map.iter()
            .map(|p| *p.1 as f64)
            .sum();

        NgramScore2 { ngram_map, total }
    }

    pub fn score(&self, s : &str) -> f64 {
        let mut score = 0.0;

        for start in 0..s.len()-NGRAM2_LEN+1 {
            let ngram = NgramText2::from(&s[start..start+NGRAM2_LEN]);

            if let Some(freq) = self.ngram_map.get(&ngram) {
                score += (*freq as f64 / self.total).log10();
            } else {
                score += (0.01 / self.total).log10();
            }
        }

        score
    }
}

impl NgramScore4 {
    pub fn create(filename: &str) -> Self {
        let file = File::open(filename)
            .expect(&format!("Cannot open {}", filename));

        let file = BufReader::new(file);

        let ngram_map : FnvHashMap<_, _> = file.lines()
            .map(|l| l.unwrap())
            .map(|l| {
                let mut it = l.split_ascii_whitespace();
                let qgram = it.next().unwrap().to_uppercase();
                let n = it.next().unwrap().parse::<i64>().unwrap();

                qgram.chars().for_each(|c| {
                    assert!('A' <= c && c <= 'Z');
                });
                assert_eq!(it.next(), None);

                (NgramText4::from(qgram.as_str()), n)
            })
            .collect();
        
        let total : f64 = ngram_map.iter()
            .map(|p| *p.1 as f64)
            .sum();

        NgramScore4 { ngram_map, total }
    }

    pub fn score(&self, s : &str) -> f64 {
        let mut score = 0.0;

        for start in 0..s.len()-NGRAM_LEN+1 {
            let ngram = NgramText4::from(&s[start..start+NGRAM_LEN]);

            if let Some(freq) = self.ngram_map.get(&ngram) {
                score += (*freq as f64 / self.total).log10();
            } else {
                score += (0.01 / self.total).log10();
            }
        }

        score
    }
}

pub struct WordListScore {
    word_set : FnvHashSet<String>,
    max_len : usize
}

impl WordListScore {
    pub fn create(filename: &str) -> Self {
        let file = File::open(filename)
            .expect(&format!("Cannot open {}", filename));

        let file = BufReader::new(file);

        let word_set : FnvHashSet<_> = file.lines()
            .map(|l| l.unwrap())
            .map(|l| {
                let mut it = l.split_ascii_whitespace();
                let word = it.next().unwrap().to_uppercase();

                word.chars().for_each(|c| {
                    assert!('A' <= c && c <= 'Z', format!("Bad: {}", word));
                });
                assert_eq!(it.next(), None);

                word
            })
            .filter(|w| (w.len() >= 2))
            .collect();

        let max_len = word_set.iter()
            .map(|w| w.len())
            .max()
            .unwrap();
        
        WordListScore { word_set, max_len }
    }

    pub fn coverage(&self, s : &str) -> f32 {
        let mut start = 0;
        let mut num_word_chars = 0;

        while start < s.len() - 1 {
            let rem = std::cmp::min(s.len() - start, self.max_len);
            for l in 0..rem {
                let word = &s[start..start+l];

                if self.word_set.contains(word) {
                    num_word_chars += word.len();
                    start += word.len() - 1;
                    break;
                }
            }

            start += 1;
        }

        num_word_chars as f32 / s.len() as f32
    }
}
