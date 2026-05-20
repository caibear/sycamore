//! Intermediate representation for `view!` macro syntax.

use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::{Expr, Ident, Path};

/// A list of nodes. This is the top-level syntax node and entry-point for parsing.
pub struct Root(pub Vec<Node>);

pub enum Node {
    /// Syntax: `<ident> ...` or `::<ident> ...`.
    Tag(TagNode),
    /// Syntax: `(<expr>)` or `"<text-expr>"`.
    Expr(Expr),
}

/// Syntax: `<ident>(<prop1>,<prop2>) { <children> }`
pub struct TagNode {
    pub ident: TagIdent,
    pub props: Vec<Prop>,
    pub children: Root,
}

pub enum TagIdent {
    /// Syntax: `<path1::path2::path3>() {}`
    Path(Path),
    /// Syntax: `<hyphenated-name>() {}`
    Custom(String),
}

impl TagIdent {
    pub fn span(&self) -> Span {
        match self {
            Self::Path(path) => path.span(),
            Self::Custom(_) => Span::call_site(),
        }
    }
}

pub struct Prop {
    pub ty: PropType,
    pub value: Expr,
    pub span: Span,
}

pub enum PropType {
    /// Syntax: `<ident>=<expr>`.
    Plain { ident: Ident },
    /// Syntax: `<hyphenated-name>=<expr>` or `"<quoted-name>"=<expr>`.
    Custom { name: String },
    /// Syntax: `<dir>:<ident>=<expr>`.
    Directive { dir: Ident, ident: Ident },
    /// Syntax: `..<expr>`
    Spread,
}
