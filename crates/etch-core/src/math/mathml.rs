use super::ir::{MathNode, MathVariant, SpaceWidth};

pub fn emit_mathml(node: &MathNode, _display: bool) -> String {
    let mut out = String::new();
    emit_node(node, &mut out);
    out
}

pub fn wrap_math_element(content: &str, display: bool) -> String {
    if display {
        format!(
            "<math xmlns=\"http://www.w3.org/1998/Math/MathML\" display=\"block\">{content}</math>"
        )
    } else {
        format!("<math xmlns=\"http://www.w3.org/1998/Math/MathML\">{content}</math>")
    }
}

fn emit_node(node: &MathNode, out: &mut String) {
    match node {
        MathNode::Number(value) => push_tag(out, "mn", value),
        MathNode::Identifier(value) => push_tag(out, "mi", value),
        MathNode::Operator(value) => push_tag(out, "mo", value),
        MathNode::Row(children) => {
            out.push_str("<mrow>");
            for child in children {
                emit_node(child, out);
            }
            out.push_str("</mrow>");
        }
        MathNode::Fraction {
            numerator,
            denominator,
        } => {
            out.push_str("<mfrac>");
            emit_node(numerator, out);
            emit_node(denominator, out);
            out.push_str("</mfrac>");
        }
        MathNode::Root { index, radicand } => {
            if let Some(index) = index {
                out.push_str("<mroot>");
                emit_node(radicand, out);
                emit_node(index, out);
                out.push_str("</mroot>");
            } else {
                out.push_str("<msqrt>");
                emit_node(radicand, out);
                out.push_str("</msqrt>");
            }
        }
        MathNode::Superscript { base, script } => {
            out.push_str("<msup>");
            emit_node(base, out);
            emit_node(script, out);
            out.push_str("</msup>");
        }
        MathNode::Subscript { base, script } => {
            out.push_str("<msub>");
            emit_node(base, out);
            emit_node(script, out);
            out.push_str("</msub>");
        }
        MathNode::SubSuperscript { base, sub, sup } => {
            out.push_str("<msubsup>");
            emit_node(base, out);
            emit_node(sub, out);
            emit_node(sup, out);
            out.push_str("</msubsup>");
        }
        MathNode::Underscript { base, under } => {
            out.push_str("<munder>");
            emit_node(base, out);
            emit_node(under, out);
            out.push_str("</munder>");
        }
        MathNode::Overscript { base, over } => {
            out.push_str("<mover>");
            emit_node(base, out);
            emit_node(over, out);
            out.push_str("</mover>");
        }
        MathNode::UnderOverscript { base, under, over } => {
            out.push_str("<munderover>");
            emit_node(base, out);
            emit_node(under, out);
            emit_node(over, out);
            out.push_str("</munderover>");
        }
        MathNode::Text(value) => push_tag(out, "mtext", value),
        MathNode::Space(width) => {
            out.push_str("<mspace width=\"");
            out.push_str(match width {
                SpaceWidth::Thin => "0.167em",
                SpaceWidth::Medium => "0.278em",
                SpaceWidth::Quad => "1em",
                SpaceWidth::DoubleQuad => "2em",
            });
            out.push_str("\"/>");
        }
        MathNode::StretchyOp(value) => {
            out.push_str("<mo stretchy=\"true\">");
            out.push_str(&escape(value));
            out.push_str("</mo>");
        }
        MathNode::Accent { base, accent } => {
            out.push_str("<mover accent=\"true\">");
            emit_node(base, out);
            push_tag(out, "mo", accent);
            out.push_str("</mover>");
        }
        MathNode::StyledIdentifier { text, variant } => {
            out.push_str("<mi mathvariant=\"");
            out.push_str(match variant {
                MathVariant::DoubleStruck => "double-struck",
                MathVariant::Script => "script",
                MathVariant::Bold => "bold",
            });
            out.push_str("\">");
            out.push_str(&escape(text));
            out.push_str("</mi>");
        }
    }
}

fn push_tag(out: &mut String, tag: &str, value: &str) {
    out.push('<');
    out.push_str(tag);
    out.push('>');
    out.push_str(&escape(value));
    out.push_str("</");
    out.push_str(tag);
    out.push('>');
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::{emit_mathml, wrap_math_element};
    use crate::math::ir::{MathNode, MathVariant, SpaceWidth};

    #[test]
    fn emits_fraction_mathml() {
        let node = MathNode::Fraction {
            numerator: Box::new(MathNode::Number("1".into())),
            denominator: Box::new(MathNode::Number("2".into())),
        };
        assert_eq!(
            emit_mathml(&node, false),
            "<mfrac><mn>1</mn><mn>2</mn></mfrac>"
        );
    }

    #[test]
    fn emits_nested_structures() {
        let node = MathNode::Fraction {
            numerator: Box::new(MathNode::Root {
                index: None,
                radicand: Box::new(MathNode::Identifier("x".into())),
            }),
            denominator: Box::new(MathNode::Superscript {
                base: Box::new(MathNode::Identifier("y".into())),
                script: Box::new(MathNode::Number("2".into())),
            }),
        };

        assert_eq!(
            emit_mathml(&node, false),
            "<mfrac><msqrt><mi>x</mi></msqrt><msup><mi>y</mi><mn>2</mn></msup></mfrac>"
        );
    }

    #[test]
    fn emits_spacing_and_styles() {
        let node = MathNode::Row(vec![
            MathNode::Space(SpaceWidth::Thin),
            MathNode::StyledIdentifier {
                text: "R".into(),
                variant: MathVariant::DoubleStruck,
            },
        ]);

        assert_eq!(
            emit_mathml(&node, false),
            "<mrow><mspace width=\"0.167em\"/><mi mathvariant=\"double-struck\">R</mi></mrow>"
        );
    }

    #[test]
    fn wraps_inline_and_display_math() {
        assert_eq!(
            wrap_math_element("<mi>x</mi>", false),
            "<math xmlns=\"http://www.w3.org/1998/Math/MathML\"><mi>x</mi></math>"
        );
        assert_eq!(
            wrap_math_element("<mi>x</mi>", true),
            "<math xmlns=\"http://www.w3.org/1998/Math/MathML\" display=\"block\"><mi>x</mi></math>"
        );
    }
}
