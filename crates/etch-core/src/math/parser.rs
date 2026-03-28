use super::{
    ir::{MathNode, MathVariant, SpaceWidth},
    lexer::{Token, tokenize},
    symbols::{Symbol, lookup_symbol},
};

pub fn parse_latex(input: &str) -> MathNode {
    let tokens = tokenize(input);
    let mut parser = MathParser::new(tokens);
    parser.parse_expression(false, false, false)
}

struct MathParser {
    tokens: Vec<Token>,
    pos: usize,
}

impl MathParser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn parse_expression(
        &mut self,
        stop_on_brace: bool,
        stop_on_bracket: bool,
        stop_on_right: bool,
    ) -> MathNode {
        let mut nodes = Vec::new();

        while let Some(token) = self.peek() {
            match token {
                Token::CloseBrace if stop_on_brace => break,
                Token::CloseBracket if stop_on_bracket => break,
                Token::Command(name) if stop_on_right && name == "right" => break,
                _ => {}
            }

            let start_pos = self.pos;
            if let Some(node) = self.parse_node() {
                nodes.push(node);
            } else if self.pos == start_pos {
                self.pos += 1;
            }
        }

        collapse_nodes(nodes)
    }

    fn parse_node(&mut self) -> Option<MathNode> {
        match self.peek()?.clone() {
            Token::Whitespace(raw) => {
                self.pos += 1;
                parse_space(&raw)
            }
            _ => {
                let base = self.parse_atom()?;
                Some(self.attach_scripts(base))
            }
        }
    }

    fn parse_atom(&mut self) -> Option<MathNode> {
        match self.peek()?.clone() {
            Token::OpenBrace => {
                self.pos += 1;
                let group = self.parse_expression(true, false, false);
                self.consume_close_brace();
                Some(group)
            }
            Token::OpenBracket => {
                self.pos += 1;
                let group = self.parse_expression(false, true, false);
                self.consume_close_bracket();
                Some(group)
            }
            Token::Text(text) => {
                self.pos += 1;
                Some(collapse_nodes(text_to_nodes(&text)))
            }
            Token::Command(name) => {
                self.pos += 1;
                self.parse_command(name)
            }
            Token::Superscript => {
                self.pos += 1;
                Some(MathNode::Operator("^".into()))
            }
            Token::Subscript => {
                self.pos += 1;
                Some(MathNode::Operator("_".into()))
            }
            Token::CloseBrace | Token::CloseBracket => None,
            Token::Whitespace(_) => None,
        }
    }

    fn parse_command(&mut self, name: String) -> Option<MathNode> {
        match name.as_str() {
            "frac" => Some(self.parse_fraction()),
            "sqrt" => Some(self.parse_root()),
            "text" => Some(self.parse_text()),
            "left" => Some(self.parse_left_right()),
            "hat" | "bar" | "vec" | "dot" | "tilde" => Some(self.parse_accent(&name)),
            "mathbb" | "mathcal" | "mathbf" => Some(self.parse_font(&name)),
            _ => lookup_symbol(&name)
                .map(symbol_to_node)
                .or_else(|| Some(MathNode::Identifier(name))),
        }
    }

    fn parse_fraction(&mut self) -> MathNode {
        let numerator = self.parse_required_group_or_atom();
        let denominator = self.parse_required_group_or_atom();
        MathNode::Fraction {
            numerator: Box::new(numerator),
            denominator: Box::new(denominator),
        }
    }

    fn parse_root(&mut self) -> MathNode {
        let index = if matches!(self.peek(), Some(Token::OpenBracket)) {
            self.pos += 1;
            let index = self.parse_expression(false, true, false);
            self.consume_close_bracket();
            Some(Box::new(index))
        } else {
            None
        };

        let radicand = self.parse_required_group_or_atom();
        MathNode::Root {
            index,
            radicand: Box::new(radicand),
        }
    }

    fn parse_text(&mut self) -> MathNode {
        MathNode::Text(self.parse_raw_group_text())
    }

    fn parse_left_right(&mut self) -> MathNode {
        let left = self.parse_delimiter();
        let inner = self.parse_expression(false, false, true);
        let right = if matches!(self.peek(), Some(Token::Command(name)) if name == "right") {
            self.pos += 1;
            self.parse_delimiter()
        } else {
            ".".to_string()
        };

        MathNode::Row(vec![
            MathNode::StretchyOp(left),
            inner,
            MathNode::StretchyOp(right),
        ])
    }

    fn parse_accent(&mut self, name: &str) -> MathNode {
        let base = self.parse_required_group_or_atom();
        MathNode::Accent {
            base: Box::new(base),
            accent: accent_symbol(name).to_string(),
        }
    }

    fn parse_font(&mut self, name: &str) -> MathNode {
        let node = self.parse_required_group_or_atom();
        let text = extract_plain_text(&node);
        MathNode::StyledIdentifier {
            text,
            variant: match name {
                "mathbb" => MathVariant::DoubleStruck,
                "mathcal" => MathVariant::Script,
                "mathbf" => MathVariant::Bold,
                _ => MathVariant::Bold,
            },
        }
    }

    fn parse_required_group_or_atom(&mut self) -> MathNode {
        while matches!(self.peek(), Some(Token::Whitespace(raw)) if raw.trim().is_empty()) {
            self.pos += 1;
        }

        if matches!(self.peek(), Some(Token::OpenBrace)) {
            self.pos += 1;
            let group = self.parse_expression(true, false, false);
            self.consume_close_brace();
            return group;
        }

        self.parse_atom()
            .map(|base| self.attach_scripts(base))
            .unwrap_or_else(|| MathNode::Identifier(String::new()))
    }

    fn parse_raw_group_text(&mut self) -> String {
        if !matches!(self.peek(), Some(Token::OpenBrace)) {
            return String::new();
        }

        self.pos += 1;
        let mut depth = 1usize;
        let mut out = String::new();

        while let Some(token) = self.peek().cloned() {
            self.pos += 1;
            match token {
                Token::OpenBrace => {
                    depth += 1;
                    if depth > 1 {
                        out.push('{');
                    }
                }
                Token::CloseBrace => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                    out.push('}');
                }
                other => out.push_str(&token_to_source(&other)),
            }
        }

        out
    }

    fn parse_delimiter(&mut self) -> String {
        match self.peek().cloned() {
            Some(Token::Text(text)) => {
                self.pos += 1;
                text
            }
            Some(Token::Command(name)) => {
                self.pos += 1;
                match lookup_symbol(&name) {
                    Some(Symbol::Identifier(value))
                    | Some(Symbol::Operator(value))
                    | Some(Symbol::LargeOperator(value)) => value.to_string(),
                    None => name,
                }
            }
            Some(Token::OpenBrace) => {
                self.pos += 1;
                "{".into()
            }
            Some(Token::CloseBrace) => {
                self.pos += 1;
                "}".into()
            }
            Some(Token::OpenBracket) => {
                self.pos += 1;
                "[".into()
            }
            Some(Token::CloseBracket) => {
                self.pos += 1;
                "]".into()
            }
            _ => ".".into(),
        }
    }

    fn attach_scripts(&mut self, base: MathNode) -> MathNode {
        let mut sub = None;
        let mut sup = None;

        loop {
            match self.peek() {
                Some(Token::Subscript) if sub.is_none() => {
                    self.pos += 1;
                    sub = Some(self.parse_script_argument());
                }
                Some(Token::Superscript) if sup.is_none() => {
                    self.pos += 1;
                    sup = Some(self.parse_script_argument());
                }
                _ => break,
            }
        }

        match (sub, sup) {
            (None, None) => base,
            (Some(sub), None) if is_limit_operator(&base) => MathNode::Underscript {
                base: Box::new(base),
                under: Box::new(sub),
            },
            (None, Some(sup)) if is_limit_operator(&base) => MathNode::Overscript {
                base: Box::new(base),
                over: Box::new(sup),
            },
            (Some(sub), Some(sup)) if is_limit_operator(&base) => MathNode::UnderOverscript {
                base: Box::new(base),
                under: Box::new(sub),
                over: Box::new(sup),
            },
            (Some(sub), None) => MathNode::Subscript {
                base: Box::new(base),
                script: Box::new(sub),
            },
            (None, Some(sup)) => MathNode::Superscript {
                base: Box::new(base),
                script: Box::new(sup),
            },
            (Some(sub), Some(sup)) => MathNode::SubSuperscript {
                base: Box::new(base),
                sub: Box::new(sub),
                sup: Box::new(sup),
            },
        }
    }

    fn consume_close_brace(&mut self) {
        if matches!(self.peek(), Some(Token::CloseBrace)) {
            self.pos += 1;
        }
    }

    fn consume_close_bracket(&mut self) {
        if matches!(self.peek(), Some(Token::CloseBracket)) {
            self.pos += 1;
        }
    }

    fn parse_script_argument(&mut self) -> MathNode {
        while matches!(self.peek(), Some(Token::Whitespace(raw)) if raw.trim().is_empty()) {
            self.pos += 1;
        }

        if matches!(self.peek(), Some(Token::OpenBrace)) {
            self.pos += 1;
            let group = self.parse_expression(true, false, false);
            self.consume_close_brace();
            return group;
        }

        self.parse_atom()
            .unwrap_or_else(|| MathNode::Identifier(String::new()))
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }
}

fn collapse_nodes(nodes: Vec<MathNode>) -> MathNode {
    match nodes.len() {
        0 => MathNode::Row(Vec::new()),
        1 => nodes.into_iter().next().expect("one node should exist"),
        _ => MathNode::Row(nodes),
    }
}

fn parse_space(raw: &str) -> Option<MathNode> {
    match raw {
        "\\," => Some(MathNode::Space(SpaceWidth::Thin)),
        "\\;" => Some(MathNode::Space(SpaceWidth::Medium)),
        "\\quad" => Some(MathNode::Space(SpaceWidth::Quad)),
        "\\qquad" => Some(MathNode::Space(SpaceWidth::DoubleQuad)),
        _ if raw.trim().is_empty() => None,
        _ => Some(MathNode::Text(raw.to_string())),
    }
}

fn text_to_nodes(text: &str) -> Vec<MathNode> {
    let mut nodes = Vec::new();
    let mut digits = String::new();

    for ch in text.chars() {
        if ch.is_ascii_digit() {
            digits.push(ch);
            continue;
        }

        if !digits.is_empty() {
            nodes.push(MathNode::Number(std::mem::take(&mut digits)));
        }

        if ch.is_ascii_alphabetic() {
            nodes.push(MathNode::Identifier(ch.to_string()));
        } else if ch.is_whitespace() {
            continue;
        } else {
            nodes.push(MathNode::Operator(ch.to_string()));
        }
    }

    if !digits.is_empty() {
        nodes.push(MathNode::Number(digits));
    }

    nodes
}

fn symbol_to_node(symbol: Symbol) -> MathNode {
    match symbol {
        Symbol::Identifier(value) => MathNode::Identifier(value.to_string()),
        Symbol::Operator(value) | Symbol::LargeOperator(value) => {
            MathNode::Operator(value.to_string())
        }
    }
}

fn token_to_source(token: &Token) -> String {
    match token {
        Token::Command(name) => format!("\\{name}"),
        Token::Text(text) => text.clone(),
        Token::OpenBrace => "{".into(),
        Token::CloseBrace => "}".into(),
        Token::OpenBracket => "[".into(),
        Token::CloseBracket => "]".into(),
        Token::Superscript => "^".into(),
        Token::Subscript => "_".into(),
        Token::Whitespace(space) => space.clone(),
    }
}

fn accent_symbol(name: &str) -> &'static str {
    match name {
        "hat" => "^",
        "bar" => "¯",
        "vec" => "→",
        "dot" => "˙",
        "tilde" => "~",
        _ => "^",
    }
}

fn extract_plain_text(node: &MathNode) -> String {
    match node {
        MathNode::Number(text)
        | MathNode::Identifier(text)
        | MathNode::Operator(text)
        | MathNode::Text(text)
        | MathNode::StretchyOp(text) => text.clone(),
        MathNode::Row(children) => children.iter().map(extract_plain_text).collect(),
        MathNode::StyledIdentifier { text, .. } => text.clone(),
        _ => String::new(),
    }
}

fn is_limit_operator(node: &MathNode) -> bool {
    matches!(
        node,
        MathNode::Operator(op)
            if matches!(op.as_str(), "∑" | "∏" | "∫" | "∬" | "∭" | "∮")
    )
}

#[cfg(test)]
mod tests {
    use super::parse_latex;
    use crate::math::ir::{MathNode, MathVariant, SpaceWidth};

    #[test]
    fn parses_simple_variable() {
        assert_eq!(parse_latex("x"), MathNode::Identifier("x".into()));
    }

    #[test]
    fn parses_fraction() {
        assert_eq!(
            parse_latex("\\frac{1}{2}"),
            MathNode::Fraction {
                numerator: Box::new(MathNode::Number("1".into())),
                denominator: Box::new(MathNode::Number("2".into())),
            }
        );
    }

    #[test]
    fn parses_greek_letters_and_operators() {
        assert_eq!(
            parse_latex("\\alpha + \\beta"),
            MathNode::Row(vec![
                MathNode::Identifier("α".into()),
                MathNode::Operator("+".into()),
                MathNode::Identifier("β".into()),
            ])
        );
    }

    #[test]
    fn parses_roots_and_scripts() {
        assert_eq!(
            parse_latex("\\sqrt[3]{x_i^2}"),
            MathNode::Root {
                index: Some(Box::new(MathNode::Number("3".into()))),
                radicand: Box::new(MathNode::SubSuperscript {
                    base: Box::new(MathNode::Identifier("x".into())),
                    sub: Box::new(MathNode::Identifier("i".into())),
                    sup: Box::new(MathNode::Number("2".into())),
                }),
            }
        );
    }

    #[test]
    fn parses_limit_operators() {
        assert_eq!(
            parse_latex("\\sum_{i=0}^{n} x_i"),
            MathNode::Row(vec![
                MathNode::UnderOverscript {
                    base: Box::new(MathNode::Operator("∑".into())),
                    under: Box::new(MathNode::Row(vec![
                        MathNode::Identifier("i".into()),
                        MathNode::Operator("=".into()),
                        MathNode::Number("0".into()),
                    ])),
                    over: Box::new(MathNode::Identifier("n".into())),
                },
                MathNode::Subscript {
                    base: Box::new(MathNode::Identifier("x".into())),
                    script: Box::new(MathNode::Identifier("i".into())),
                },
            ])
        );
    }

    #[test]
    fn parses_text_accents_fonts_and_spacing() {
        assert_eq!(
            parse_latex("\\text{if } \\vec{v} \\qquad \\mathbb{R}"),
            MathNode::Row(vec![
                MathNode::Text("if ".into()),
                MathNode::Accent {
                    base: Box::new(MathNode::Identifier("v".into())),
                    accent: "→".into(),
                },
                MathNode::Space(SpaceWidth::DoubleQuad),
                MathNode::StyledIdentifier {
                    text: "R".into(),
                    variant: MathVariant::DoubleStruck,
                },
            ])
        );
    }

    #[test]
    fn parses_dot_accent_with_correct_symbol() {
        assert_eq!(
            parse_latex("\\dot{x}"),
            MathNode::Accent {
                base: Box::new(MathNode::Identifier("x".into())),
                accent: "˙".into(),
            }
        );
    }

    #[test]
    fn parses_left_right_delimiters() {
        assert_eq!(
            parse_latex("\\left(\\frac{1}{2}\\right)"),
            MathNode::Row(vec![
                MathNode::StretchyOp("(".into()),
                MathNode::Fraction {
                    numerator: Box::new(MathNode::Number("1".into())),
                    denominator: Box::new(MathNode::Number("2".into())),
                },
                MathNode::StretchyOp(")".into()),
            ])
        );
    }
}
