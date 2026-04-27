#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    pub line: usize,
    pub col: usize,
}

impl Span {
    pub fn new(line: usize, col: usize) -> Self {
        Span { line, col }
    }
}

#[derive(Debug, Clone)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(node: T, span: Span) -> Self {
        Spanned { node, span }
    }
}

pub type Expr = Spanned<ExprKind>;
pub type Stmt = Spanned<StmtKind>;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Float,
    Str,
    Flag,
    Byte,
    Void,
    Array(Box<Type>),
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    IntLit(i64),
    FloatLit(f64),
    StrLit(String),
    BoolLit(bool),
    Air,

    Ident(String),

    Binary {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },

    Unary {
        op: UnOp,
        expr: Box<Expr>,
    },

    // fire ident(args)
    Call {
        name: String,
        args: Vec<Expr>,
    },

    // arr[index]
    Index {
        array: Box<Expr>,
        index: Box<Expr>,
    },

    // [a, b, c]
    ArrayLit(Vec<Expr>),

    // start..end  or  start..=end
    Range {
        start: Box<Expr>,
        end: Box<Expr>,
        inclusive: bool,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnOp {
    Neg,
    Not,
}

#[derive(Debug, Clone)]
pub enum StmtKind {
    // payload x: int = expr   (mutable=false)
    // fuel    x: int = expr   (mutable=true)
    Let {
        mutable: bool,
        name: String,
        ty: Type,
        init: Expr,
    },

    // x = expr  /  x += expr  /  arr[i] = expr  ...
    Assign {
        target: AssignTarget,
        op: AssignOp,
        value: Expr,
    },

    // scan expr { ... } fallback { ... }
    If {
        cond: Expr,
        then_block: Vec<Stmt>,
        else_block: Option<Vec<Stmt>>,
    },

    // route expr { pat => stmt ... }
    Route {
        subject: Expr,
        arms: Vec<RouteArm>,
    },

    // burn expr { ... }
    Burn {
        cond: Expr,
        body: Vec<Stmt>,
    },

    // spin i in range { ... }
    Spin {
        var: String,
        range: Expr,
        body: Vec<Stmt>,
    },

    // orbit { ... }
    Orbit {
        body: Vec<Stmt>,
    },

    // fire ident(args)  — call used as a statement
    Fire {
        name: String,
        args: Vec<Expr>,
    },

    Land(Option<Expr>),
    Eject,
    Pass,
    Abort,
}

#[derive(Debug, Clone)]
pub enum AssignTarget {
    Ident(String),
    Index { name: String, index: Box<Expr> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssignOp {
    Assign,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

#[derive(Debug, Clone)]
pub struct RouteArm {
    pub pattern: Expr,
    pub body: Box<Stmt>,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub enum Item {
    // stage name(params) lands type { body }
    Stage {
        name: String,
        params: Vec<Param>,
        ret_type: Type,
        body: Vec<Stmt>,
    },

    // launch name { body }
    Launch {
        name: String,
        body: Vec<Stmt>,
    },
}

#[derive(Debug)]
pub struct Program {
    pub ignite_body: Vec<Stmt>,
    pub items: Vec<Item>,
}
