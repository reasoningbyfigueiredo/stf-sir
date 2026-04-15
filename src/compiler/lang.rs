//! FEAT-207-3: Language detection — BCP-47 language tag detection.
//!
//! Implements a heuristic language detector using common stopwords for
//! Portuguese and English. Returns `"und"` when confidence < 0.7.
//!
//! No external dependencies are required; this is a pure-Rust implementation
//! suitable for the minimal binary target (feature-free).

// ---------------------------------------------------------------------------
// Output type
// ---------------------------------------------------------------------------

/// The result of a language detection attempt.
#[derive(Debug, Clone, PartialEq)]
pub struct LanguageDetection {
    /// BCP-47 language tag (e.g. `"en"`, `"pt"`) or `"und"` if undetermined.
    pub tag: String,
    /// Detection confidence in the range `0.0..=1.0`.
    pub confidence: f32,
}

impl LanguageDetection {
    /// Construct an undetermined result with the given confidence.
    pub fn undetermined(confidence: f32) -> Self {
        Self {
            tag: "und".to_string(),
            confidence,
        }
    }

    /// Returns `true` if the language could not be determined.
    pub fn is_undetermined(&self) -> bool {
        self.tag == "und"
    }
}

// ---------------------------------------------------------------------------
// Stopword lists
// ---------------------------------------------------------------------------

const EN_STOPWORDS: &[&str] = &[
    "the", "and", "that", "have", "for", "not", "with", "this", "but",
    "from", "they", "this", "will", "would", "there", "their", "what",
    "about", "which", "when", "make", "like", "time", "just", "know",
    "take", "people", "into", "year", "your", "good", "some", "could",
    "them", "other", "than", "then", "look", "only", "come", "over",
    "think", "also", "back", "after", "use", "two", "how", "our",
    "work", "first", "well", "even", "want", "because", "any", "these",
    "give", "most", "does", "does", "been", "were", "said",
];

const PT_STOPWORDS: &[&str] = &[
    "que", "não", "uma", "para", "com", "por", "mais", "como", "mas",
    "foi", "ele", "das", "tem", "aos", "seu", "sua", "ou", "ser",
    "quando", "muito", "há", "nos", "já", "está", "também", "pelo",
    "pela", "até", "isso", "ela", "entre", "era", "depois", "sem",
    "mesmo", "aos", "ter", "seus", "quem", "nas", "me", "esse",
    "eles", "estão", "você", "tinha", "foram", "essa", "num", "nem",
    "suas", "meu", "às", "minha", "têm", "numa", "pelos", "elas",
    "havia", "seja", "qual", "será", "nós", "tenho", "lhe", "deles",
];

// ---------------------------------------------------------------------------
// Detection function
// ---------------------------------------------------------------------------

/// Detect the BCP-47 language tag of `text` using a stopword heuristic.
///
/// The detector counts matches against English and Portuguese stopword lists.
/// If the winning language's normalised score is ≥ 0.7 (i.e. at least 70% of
/// detected stopwords belong to one language), it is returned with that
/// confidence. Otherwise `"und"` is returned.
///
/// For very short texts (< 10 words) the minimum confidence threshold is
/// raised proportionally to reduce false positives.
///
/// # Examples
///
/// ```no_run
/// // Use via stf_sir::sir after integration
/// // let d = detect_language("The quick brown fox jumps over the lazy dog");
/// // assert_eq!(d.tag, "en");
/// ```
pub fn detect_language(text: &str) -> LanguageDetection {
    let tokens: Vec<String> = text
        .split(|c: char| !c.is_alphabetic())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_lowercase())
        .collect();

    if tokens.is_empty() {
        return LanguageDetection::undetermined(0.0);
    }

    let en_hits = tokens
        .iter()
        .filter(|t| EN_STOPWORDS.contains(&t.as_str()))
        .count();

    let pt_hits = tokens
        .iter()
        .filter(|t| PT_STOPWORDS.contains(&t.as_str()))
        .count();

    let total_hits = en_hits + pt_hits;

    if total_hits == 0 {
        return LanguageDetection::undetermined(0.0);
    }

    let (winner_tag, winner_hits) = if en_hits >= pt_hits {
        ("en", en_hits)
    } else {
        ("pt", pt_hits)
    };

    let raw_confidence = winner_hits as f32 / total_hits as f32;

    // For short texts, require higher relative dominance to avoid false positives
    let length_penalty = if tokens.len() < 20 {
        0.05 * (20usize.saturating_sub(tokens.len()) as f32 / 20.0)
    } else {
        0.0
    };

    let confidence = (raw_confidence - length_penalty).max(0.0).min(1.0);

    if confidence >= 0.7 {
        LanguageDetection {
            tag: winner_tag.to_string(),
            confidence,
        }
    } else {
        LanguageDetection::undetermined(confidence)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_english() {
        let text = "The quick brown fox jumps over the lazy dog. \
                    This is a simple English sentence with some common words.";
        let result = detect_language(text);
        assert_eq!(result.tag, "en", "expected English, got {:?}", result);
        assert!(result.confidence >= 0.7);
    }

    #[test]
    fn detects_portuguese() {
        let text = "Este é um texto em português com algumas palavras muito comuns. \
                    Quando não há mais tempo para esperar, temos que agir.";
        let result = detect_language(text);
        assert_eq!(result.tag, "pt", "expected Portuguese, got {:?}", result);
        assert!(result.confidence >= 0.7);
    }

    #[test]
    fn empty_text_returns_und() {
        let result = detect_language("");
        assert!(result.is_undetermined());
    }

    #[test]
    fn unknown_language_returns_und() {
        // Finnish text — neither English nor Portuguese stopwords
        let result = detect_language("Hyvää huomenta Suomi on kaunis maa talvella");
        assert!(result.is_undetermined());
    }
}
