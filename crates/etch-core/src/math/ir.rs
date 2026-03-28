#[derive(Debug, Clone, PartialEq)]
pub enum MathNode {
    Number(String),
    Identifier(String),
    Operator(String),
    Row(Vec<MathNode>),
    Fraction {
        numerator: Box<MathNode>,
        denominator: Box<MathNode>,
    },
    Root {
        index: Option<Box<MathNode>>,
        radicand: Box<MathNode>,
    },
    Superscript {
        base: Box<MathNode>,
        script: Box<MathNode>,
    },
    Subscript {
        base: Box<MathNode>,
        script: Box<MathNode>,
    },
    SubSuperscript {
        base: Box<MathNode>,
        sub: Box<MathNode>,
        sup: Box<MathNode>,
    },
    Underscript {
        base: Box<MathNode>,
        under: Box<MathNode>,
    },
    Overscript {
        base: Box<MathNode>,
        over: Box<MathNode>,
    },
    UnderOverscript {
        base: Box<MathNode>,
        under: Box<MathNode>,
        over: Box<MathNode>,
    },
    Text(String),
    Space(SpaceWidth),
    StretchyOp(String),
    Accent {
        base: Box<MathNode>,
        accent: String,
    },
    StyledIdentifier {
        text: String,
        variant: MathVariant,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpaceWidth {
    Thin,
    Medium,
    Quad,
    DoubleQuad,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MathVariant {
    DoubleStruck,
    Script,
    Bold,
}
