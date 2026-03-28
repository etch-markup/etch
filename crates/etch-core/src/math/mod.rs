pub mod ir;
pub mod lexer;
pub mod mathml;
pub mod parser;
pub mod symbols;

pub fn latex_to_mathml(input: &str, display: bool) -> String {
    let node = parser::parse_latex(input);
    let inner = mathml::emit_mathml(&node, display);
    mathml::wrap_math_element(&inner, display)
}

#[cfg(test)]
mod tests {
    use super::latex_to_mathml;

    #[test]
    fn end_to_end_fraction() {
        assert_eq!(
            latex_to_mathml("\\frac{1}{2}", false),
            "<math xmlns=\"http://www.w3.org/1998/Math/MathML\"><mfrac><mn>1</mn><mn>2</mn></mfrac></math>"
        );
    }

    #[test]
    fn end_to_end_integral_display() {
        let result = latex_to_mathml("\\int_0^1 x^2 \\, dx", true);
        assert!(
            result.starts_with(
                "<math xmlns=\"http://www.w3.org/1998/Math/MathML\" display=\"block\">"
            )
        );
        assert!(result.contains("<munderover>") || result.contains("<munder>"));
    }
}
