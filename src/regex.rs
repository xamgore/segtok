use fancy_regex::{Matches, Regex};

#[derive(Debug, Copy, Clone)]
pub enum Partition<'s> {
    Match(&'s str),
    NonMatch(&'s str),
}

impl<'s> Partition<'s> {
    pub fn into_inner(self) -> &'s str {
        match self {
            Partition::Match(str) => str,
            Partition::NonMatch(str) => str,
        }
    }

    pub fn into_pair(self) -> (&'s str, bool) {
        (self.into_inner(), matches!(self, Partition::Match(_)))
    }
}

impl<'s> From<Partition<'s>> for &'s str {
    fn from(value: Partition<'s>) -> Self {
        value.into_inner()
    }
}

impl<'s> AsRef<str> for Partition<'s> {
    fn as_ref(&self) -> &str {
        self.into_inner()
    }
}

/// ```ignore
/// let re = Regex::new(r"\d+").unwrap();
/// let text = "123abcdef456ghj789";
/// for part in PartitionIter::new(&re, text) {
///     dbg!(part);
/// }
/// ```
#[derive(Debug)]
pub struct PartitionIter<'r, 't> {
    it: Matches<'r, 't>,
    last_match_end: usize,
    text: &'t str,
    next_match: Option<&'t str>,
}

impl<'r, 't> PartitionIter<'r, 't> {
    pub fn new(re: &'r Regex, text: &'t str) -> PartitionIter<'r, 't> {
        PartitionIter { it: re.find_iter(text), last_match_end: 0, text, next_match: None }
    }
}

impl<'t> Iterator for PartitionIter<'_, 't> {
    type Item = Partition<'t>;

    fn next(&mut self) -> Option<Partition<'t>> {
        if let Some(next_match) = self.next_match.take() {
            return Some(Partition::Match(next_match));
        }
        match self.it.next().map(Result::unwrap) {
            None => {
                if self.last_match_end >= self.text.len() {
                    None
                } else {
                    let non_match = &self.text[self.last_match_end..];
                    self.last_match_end = self.text.len();
                    Some(Partition::NonMatch(non_match))
                }
            }
            Some(m) => {
                if m.start() > self.last_match_end {
                    let non_match = &self.text[self.last_match_end..m.start()];
                    self.last_match_end = m.end();
                    self.next_match = Some(m.as_str());
                    Some(Partition::NonMatch(non_match))
                } else {
                    self.last_match_end = m.end();
                    Some(Partition::Match(m.as_str()))
                }
            }
        }
    }
}

pub trait RegexSplitExt {
    /// Split `target` by the occurrences of regex pattern.
    /// The text of all groups in the pattern are also returned as part of the resulting list.
    fn split_with_separators<'r, 'h>(&'r self, target: &'h str) -> impl Iterator<Item = &'h str> + Sized;
}

impl RegexSplitExt for Regex {
    fn split_with_separators<'r, 'h>(&'r self, target: &'h str) -> impl Iterator<Item = &'h str> + Sized {
        PartitionIter::new(self, target).map(Partition::into_inner)
    }
}
