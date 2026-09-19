use serde::Serialize;

/// Mutually exclusive physical-line categories. Mixed code/comment lines are code.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Stats {
    pub files: u64,
    pub lines: u64,
    pub code: u64,
    pub comments: u64,
    pub blanks: u64,
    pub unclassified: u64,
    pub bytes: u64,
}
impl Stats {
    pub fn non_blank(self) -> u64 { self.lines.saturating_sub(self.blanks) }
    pub fn is_consistent(self) -> bool {
        self.lines == self.code + self.comments + self.blanks + self.unclassified
    }
    pub fn add(&mut self, rhs: Self) {
        self.files += rhs.files;
        self.lines += rhs.lines;
        self.code += rhs.code;
        self.comments += rhs.comments;
        self.blanks += rhs.blanks;
        self.unclassified += rhs.unclassified;
        self.bytes += rhs.bytes;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AreaMetric { #[default] NonBlank, Lines, Code, Bytes }
impl AreaMetric {
    pub fn next(self) -> Self {
        match self { Self::NonBlank => Self::Lines, Self::Lines => Self::Code, Self::Code => Self::Bytes, Self::Bytes => Self::NonBlank }
    }
    pub fn label(self) -> &'static str {
        match self { Self::NonBlank => "Non-blank lines", Self::Lines => "Physical lines", Self::Code => "Code lines", Self::Bytes => "Text bytes" }
    }
    pub fn value(self, stats: Stats) -> u64 {
        match self { Self::NonBlank => stats.non_blank(), Self::Lines => stats.lines, Self::Code => stats.code, Self::Bytes => stats.bytes }
    }
}
