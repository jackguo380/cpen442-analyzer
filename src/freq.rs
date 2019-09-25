use std::cmp::Ordering;

const ALPHABET : &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

const ENGLFREQS : &str = "ETAOINSRHDLUCMFYWGPBVKXQJZ";

pub struct Freq {
    pub freqs :  Vec<(char, u32)>,
    pub score : u32
}

impl Freq {
    pub fn calc_score(freqs: &[(char, u32)]) -> u32 {
        let mut score : u32 = 0;
        let mut multiplier = ENGLFREQS.len() as u32;

        assert_eq!(ENGLFREQS.len(), freqs.len());

        for p in freqs {
            score += (ENGLFREQS.len() - ENGLFREQS.find(p.0).unwrap()) as u32 * multiplier;
            multiplier -= 1;
        }

        score
    }
}


impl PartialEq for Freq {
    fn eq(&self, other: &Self) -> bool {
        (self.score == other.score)
    }
}

impl Eq for Freq {}

impl PartialOrd for Freq {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.score > other.score {
            Some(Ordering::Greater)
        } else if self.score < other.score {
            Some(Ordering::Less)
        } else {
            Some(Ordering::Equal)
        }
    }
}

impl Ord for Freq {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.score > other.score {
            Ordering::Greater
        } else if self.score < other.score {
            Ordering::Less
        } else {
            Ordering::Equal
        }
    }
}

impl From<&str> for Freq {
    fn from(text : &str) -> Self {
        let mut freqs = Vec::<(char, u32)>::with_capacity(26);

        for c in ENGLFREQS.chars() {
            freqs.push((c, text.chars().filter(|v| c == *v).count() as u32));
        }

        &mut freqs.sort_by(|p1, p2| p2.1.cmp(&p1.1));

        let score = Freq::calc_score(&freqs);

        Freq { freqs, score }
    }
}

impl std::fmt::Display for Freq {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "Freq {{")?;

        let mut count = 0;
        for p in &self.freqs {
            write!(f, "({}, {}) ", p.0, p.1)?;

            count += 1;
            if count == 4 {
                write!(f, "\n")?;
                count = 0
            }
        }

        writeln!(f, "score =  {}", self.score)?;
        writeln!(f, "}}")?;

        Ok(())
    }

}
