use inner::NumberChecker;
use keyword::{KeywordScore, KeywordSort};
use regex::Regex;
use std::collections::HashMap;
use stopwords::StopWords;

/// Represents an instance of Rake type
#[derive(Debug, Clone)]
pub struct Rake {
    stop_words: StopWords,
    num_re: Regex,
    punc_re: Regex,
}

impl Rake {
    /// Create a new instance of `Rake`.
    /// `stop_words` is an instance of `StopWords` struct.
    pub fn new(stop_words: StopWords) -> Self {
        Rake {
            stop_words: stop_words,
            num_re: Regex::new(r"-?\p{N}+[./٫,']?\p{N}*").expect("bad regex"),
            punc_re: Regex::new(r"[^\P{P}-]|\s+-\s+").expect("bad regex"),
        }
    }

    /// Runs RAKE algorithm on `text` and returns a vector of keywords.
    /// The returned vector is sorted by score (from greater to less).
    pub fn run(&self, text: &str) -> Vec<KeywordScore> {
        let phrases = self.phrases(self.punc_re.split(text));
        let word_scores = self.word_scores(&phrases);
        self.candidate_keywords(&phrases, word_scores)
    }

    /// Runs RAKE algorithm on chunks of text (such as sentences), and returns a vector of keywords.
    /// The returned vector is sorted by score (from greater to less).
    #[inline]
    pub fn run_fragments<'a>(
        &self,
        fragments: impl IntoIterator<Item = &'a str>,
    ) -> Vec<KeywordScore> {
        let phrases = self.phrases(fragments);
        let word_scores = self.word_scores(&phrases);
        self.candidate_keywords(&phrases, word_scores)
    }

    fn candidate_keywords<'a>(
        &self,
        phrases: &[Vec<&'a str>],
        word_scores: HashMap<&'a str, f64>,
    ) -> Vec<KeywordScore> {
        let mut keyword_score = HashMap::with_capacity(phrases.len());
        phrases.iter().for_each(|phrase| {
            let mut candidate_score = 0f64;
            phrase
                .iter()
                .filter(|word| !self.is_number(word))
                .for_each(|word| candidate_score += word_scores[word]);
            *keyword_score.entry(phrase.join(" ")).or_insert(0f64) = candidate_score;
        });
        let mut keywords = KeywordScore::from_map(keyword_score);
        keywords.sort_by_score();
        keywords
    }

    fn word_scores<'a>(&self, phrases: &[Vec<&'a str>]) -> HashMap<&'a str, f64> {
        let mut word_freq = HashMap::new();
        let mut word_degree = HashMap::new();
        phrases.iter().for_each(|phrase| {
            let len: usize = phrase
                .iter()
                .map(|word| if self.is_number(word) { 0 } else { 1 })
                .sum();
            if len > 0 {
                phrase
                    .iter()
                    .filter(|word| !self.is_number(word))
                    .for_each(|word| {
                        *word_freq.entry(*word).or_insert(0) += 1;
                        *word_degree.entry(*word).or_insert(0) += len - 1;
                    });
            }
        });
        let mut word_score = HashMap::new();
        for (word, freq) in word_freq {
            word_score
                .entry(word)
                .or_insert((word_degree[word] + freq) as f64 / freq as f64);
        }
        word_score
    }

    fn phrases<'a>(&self, phrases_iter: impl IntoIterator<Item = &'a str>) -> Vec<Vec<&'a str>> {
        let phrases_iter = phrases_iter.into_iter();
        let mut phrases = Vec::with_capacity(2 * phrases_iter.size_hint().0);
        for s in phrases_iter.filter(|s| !s.is_empty()) {
            let mut phrase = Vec::new();
            for word in s.split_whitespace() {
                let word = word.trim_matches(|c: char| !c.is_alphanumeric());
                if self.stop_words.contains(&word.to_lowercase()) {
                    if !phrase.is_empty() {
                        phrases.push(phrase.clone());
                        phrase.clear();
                    }
                } else {
                    phrase.push(word);
                }
            }
            if !phrase.is_empty() {
                phrases.push(phrase);
            }
        }
        phrases
    }
}

impl NumberChecker<&str> for &crate::Rake {
    fn is_number(&self, s: &str) -> bool {
        self.num_re.is_match(s)
    }
}
