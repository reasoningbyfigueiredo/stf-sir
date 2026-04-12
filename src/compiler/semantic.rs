use unicode_normalization::UnicodeNormalization;

use crate::model::ZToken;

pub fn apply(ztokens: &mut [ZToken]) {
    for token in ztokens {
        let normalized_text = normalize_text(&token.lexical.plain_text);
        token.lexical.normalized_text = normalized_text.clone();
        token.semantic.gloss = if token.lexical.plain_text.is_empty() {
            String::new()
        } else {
            normalized_text
        };
    }
}

pub(crate) fn normalize_text(input: &str) -> String {
    let normalized = input.nfkc().collect::<String>();
    normalized.split_whitespace().collect::<Vec<_>>().join(" ")
}
