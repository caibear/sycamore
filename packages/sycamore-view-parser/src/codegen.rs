//! Codegen for `view!` macro.
//!
//! Implementation note: We are not using the `quote::ToTokens` trait because we need to pass
//! additional information to the codegen such as which mode (Client, Hydrate, SSR), etc...

use proc_macro2::TokenStream;
use quote::quote;

use crate::ir::{Node, Prop, PropType, Root, TagIdent, TagNode};

pub struct Codegen {
    // TODO: configure mode: Client, Hydrate, SSR
}

impl Codegen {
    pub fn root(&self, root: &Root) -> TokenStream {
        match &root.0[..] {
            [] => quote! {
                ::sycamore::rt::View::new()
            },
            [node] => self.node(node),
            nodes => {
                let nodes = nodes.iter().map(|node| self.node(node));
                quote! {
                    ::std::convert::Into::<::sycamore::rt::View>::into(::std::vec![#(#nodes),*])
                }
            }
        }
    }

    /// Generate a `View` from a `Node`.
    pub fn node(&self, node: &Node) -> TokenStream {
        match node {
            Node::Tag(tag) => {
                if is_component(&tag.ident) {
                    self.component(tag)
                } else {
                    self.element(tag)
                }
            }
            Node::Expr(expr) => quote! {
                ::std::convert::Into::<::sycamore::rt::View>::into(#expr)
            }
        }
    }

    pub fn element(&self, element: &TagNode) -> TokenStream {
        let TagNode {
            ident,
            props,
            children,
        } = element;

        let attributes = props.iter().map(|attr| self.attribute(attr));

        let children = children
            .0
            .iter()
            .map(|child| self.node(child))
            .collect::<Vec<_>>();

        match ident {
            TagIdent::Path(tag) => {
                assert!(tag.get_ident().is_some(), "elements must be an ident");
                quote! {
                    ::sycamore::rt::View::from(
                        ::sycamore::rt::tags::#tag().children(::std::vec![#(#children),*])#(#attributes)*
                    )
                }
            }
            TagIdent::Custom(tag) => quote! {
                ::sycamore::rt::View::from(
                    ::sycamore::rt::custom_element(#tag).children(::std::vec![#(#children),*])#(#attributes)*
                )
            },
        }
    }

    pub fn attribute(&self, attr: &Prop) -> TokenStream {
        let value = &attr.value;
        match &attr.ty {
            PropType::Plain { ident } => {
                quote! { .#ident(#value) }
            }
            PropType::Custom { name: ident } => {
                quote! { .attr(#ident, #value) }
            }
            PropType::Directive { dir, ident } => match dir.to_string().as_str() {
                "on" => quote! { .on(::sycamore::rt::events::#ident, #value) },
                "prop" => {
                    let ident = ident.to_string();
                    quote! { .prop(#ident, #value) }
                }
                "bind" => quote! { .bind(::sycamore::rt::bind::#ident, #value) },
                _ => syn::Error::new(dir.span(), format!("unknown directive `{dir}`"))
                    .to_compile_error(),
            },
            PropType::Spread => quote! { .spread(#value) },
        }
    }

    pub fn component(
        &self,
        TagNode {
            ident,
            props,
            children,
        }: &TagNode,
    ) -> TokenStream {
        let ident = match ident {
            TagIdent::Path(path) => path,
            TagIdent::Custom(_) => unreachable!("hyphenated tags are not components"),
        };

        let plain = props
            .iter()
            .filter_map(|prop| match &prop.ty {
                PropType::Plain { ident } => Some((ident, prop.value.clone())),
                _ => None,
            })
            .collect::<Vec<_>>();
        let plain_names = plain.iter().map(|(ident, _)| ident);
        let plain_values = plain.iter().map(|(_, value)| value);

        let other_props = props
            .iter()
            .filter(|prop| !matches!(&prop.ty, PropType::Plain { .. }))
            .collect::<Vec<_>>();
        let other_attributes = other_props.iter().map(|prop| self.attribute(prop));

        let children_quoted = if children.0.is_empty() {
            quote! {}
        } else {
            let codegen = Codegen {};
            let children = codegen.root(children);
            quote! {
                .children(
                    ::sycamore::rt::Children::new(move || {
                        #children
                    })
                )
            }
        };
        quote! {{
            let __component = &#ident; // We do this to make sure the compiler can infer the value for `<G>`.
            ::sycamore::rt::component_scope(move || ::sycamore::rt::Component::create(
                __component,
                ::sycamore::rt::element_like_component_builder(__component)
                    #(.#plain_names(#plain_values))*
                    #(#other_attributes)*
                    #children_quoted
                    .build()
            ))
        }}
    }
}

fn is_component(ident: &TagIdent) -> bool {
    match ident {
        TagIdent::Path(path) => {
            path.get_ident().is_none()
                || path
                    .get_ident()
                    .unwrap()
                    .to_string()
                    .chars()
                    .next()
                    .unwrap()
                    .is_ascii_uppercase()
        }
        // A hyphenated tag is always a custom-element and therefore never a component.
        TagIdent::Custom(_) => false,
    }
}
