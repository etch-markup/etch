use crate::Inline;

#[allow(dead_code)]
pub(crate) fn parse_inlines(input: &str) -> Vec<Inline> {
    vec![Inline::Text {
        value: input.to_string(),
    }]
}
