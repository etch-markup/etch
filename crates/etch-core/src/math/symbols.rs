#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Symbol {
    Identifier(&'static str),
    Operator(&'static str),
    LargeOperator(&'static str),
}

pub fn lookup_symbol(name: &str) -> Option<Symbol> {
    match name {
        "alpha" => Some(Symbol::Identifier("α")),
        "beta" => Some(Symbol::Identifier("β")),
        "gamma" => Some(Symbol::Identifier("γ")),
        "delta" => Some(Symbol::Identifier("δ")),
        "epsilon" => Some(Symbol::Identifier("ε")),
        "theta" => Some(Symbol::Identifier("θ")),
        "lambda" => Some(Symbol::Identifier("λ")),
        "mu" => Some(Symbol::Identifier("μ")),
        "pi" => Some(Symbol::Identifier("π")),
        "sigma" => Some(Symbol::Identifier("σ")),
        "omega" => Some(Symbol::Identifier("ω")),
        "Gamma" => Some(Symbol::Identifier("Γ")),
        "Delta" => Some(Symbol::Identifier("Δ")),
        "Theta" => Some(Symbol::Identifier("Θ")),
        "Lambda" => Some(Symbol::Identifier("Λ")),
        "Pi" => Some(Symbol::Identifier("Π")),
        "Sigma" => Some(Symbol::Identifier("Σ")),
        "Omega" => Some(Symbol::Identifier("Ω")),
        "sum" => Some(Symbol::LargeOperator("∑")),
        "prod" => Some(Symbol::LargeOperator("∏")),
        "int" => Some(Symbol::LargeOperator("∫")),
        "iint" => Some(Symbol::LargeOperator("∬")),
        "iiint" => Some(Symbol::LargeOperator("∭")),
        "oint" => Some(Symbol::LargeOperator("∮")),
        "pm" => Some(Symbol::Operator("±")),
        "times" => Some(Symbol::Operator("×")),
        "div" => Some(Symbol::Operator("÷")),
        "cdot" => Some(Symbol::Operator("⋅")),
        "leq" => Some(Symbol::Operator("≤")),
        "geq" => Some(Symbol::Operator("≥")),
        "neq" => Some(Symbol::Operator("≠")),
        "approx" => Some(Symbol::Operator("≈")),
        "equiv" => Some(Symbol::Operator("≡")),
        "in" => Some(Symbol::Operator("∈")),
        "notin" => Some(Symbol::Operator("∉")),
        "subset" => Some(Symbol::Operator("⊂")),
        "cup" => Some(Symbol::Operator("∪")),
        "cap" => Some(Symbol::Operator("∩")),
        "to" | "rightarrow" => Some(Symbol::Operator("→")),
        "leftarrow" => Some(Symbol::Operator("←")),
        "Rightarrow" => Some(Symbol::Operator("⇒")),
        "Leftrightarrow" => Some(Symbol::Operator("⇔")),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{Symbol, lookup_symbol};

    #[test]
    fn looks_up_greek_letters() {
        assert_eq!(lookup_symbol("alpha"), Some(Symbol::Identifier("α")));
        assert_eq!(lookup_symbol("Omega"), Some(Symbol::Identifier("Ω")));
        assert_eq!(lookup_symbol("nonexistent"), None);
    }

    #[test]
    fn looks_up_operators_and_arrows() {
        assert_eq!(lookup_symbol("pm"), Some(Symbol::Operator("±")));
        assert_eq!(lookup_symbol("times"), Some(Symbol::Operator("×")));
        assert_eq!(lookup_symbol("leq"), Some(Symbol::Operator("≤")));
        assert_eq!(lookup_symbol("to"), Some(Symbol::Operator("→")));
        assert_eq!(lookup_symbol("Rightarrow"), Some(Symbol::Operator("⇒")));
        assert_eq!(lookup_symbol("Leftrightarrow"), Some(Symbol::Operator("⇔")));
    }

    #[test]
    fn looks_up_large_operators() {
        assert_eq!(lookup_symbol("sum"), Some(Symbol::LargeOperator("∑")));
        assert_eq!(lookup_symbol("int"), Some(Symbol::LargeOperator("∫")));
    }
}
