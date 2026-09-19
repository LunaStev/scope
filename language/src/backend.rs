//! The only adapter to the pinned upstream language catalogue.
use tokei::LanguageType;
#[derive(Clone, Copy, Debug)]
pub(crate) enum Engine { Wave, Tokei(LanguageType), Plain }
impl Engine {
    pub fn named(name: &str) -> Result<Self, String> {
        match name {
            "wave" => Ok(Self::Wave),
            "plain" => Ok(Self::Plain),
            name => name.parse::<LanguageType>().ok().or_else(|| LanguageType::list().iter().copied().find(|lang|format!("{lang:?}").eq_ignore_ascii_case(name)))
                .map(Self::Tokei).ok_or_else(||format!("Unknown classifier: {name}")),
        }
    }
}
pub(crate) fn extension(ext: &str) -> Option<LanguageType> {
    // Preserve meaningful case (.C); never reopen the file for detection.
    LanguageType::from_file_extension(ext)
        .or_else(|| LanguageType::from_file_extension(&ext.to_ascii_lowercase()))
}
