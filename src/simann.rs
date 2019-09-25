use std::fmt;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};

#[derive(Default)]
#[derive(Clone)]
pub struct SimulatedAnnResult {
    pub key : String,
    pub decrypt : String,
    pub score : f64,
    pub word_coverage : f32,
}

impl PartialEq for SimulatedAnnResult {
    fn eq(&self, other: &Self) -> bool {
        (self.score == other.score)
    }
}

impl Eq for SimulatedAnnResult {}

impl PartialOrd for SimulatedAnnResult {
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

impl Ord for SimulatedAnnResult {
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

impl Display for SimulatedAnnResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "SimAnnResult ( Key = {} Score = {} Coverage = {} Decrypt = {} )",
            self.key, self.score, self.word_coverage, self.decrypt)
    }
}

