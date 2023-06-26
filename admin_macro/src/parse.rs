use proc_macro2::Span;
use syn::{Expr, ExprLit, ExprPath, Ident, Lit, MetaNameValue};

type Result<T> = std::result::Result<T, syn::Error>;

#[derive(Debug, Clone)]
pub enum IdentOrLiteral {
    Path(ExprPath),
    Literal(ExprLit),
}

fn parse_path_ident<'a>(
    ident: &'a Ident,
    nv: &'a MetaNameValue,
    error_message: &'a str,
) -> Result<&'a Ident> {
    match &nv.value {
        syn::Expr::Path(p) => {
            if let Some(i) = p.path.get_ident() {
                return Ok(i);
            } else {
                Err(syn::Error::new(ident.span(), error_message))
            }
        }
        _ => Err(syn::Error::new(ident.span(), error_message)),
    }
}

fn parse_path_ident_from_expr<'a>(
    ident: &'a Ident,
    nv: &'a Expr,
    error_message: &'a str,
) -> Result<&'a Ident> {
    match &nv {
        syn::Expr::Path(p) => {
            if let Some(i) = p.path.get_ident() {
                return Ok(i);
            } else {
                Err(syn::Error::new(ident.span(), error_message))
            }
        }
        _ => Err(syn::Error::new(ident.span(), error_message)),
    }
}

fn parse_expr_path<'a>(
    ident: &'a Ident,
    nv: &'a MetaNameValue,
    error_message: &'a str,
) -> Result<&'a ExprPath> {
    match &nv.value {
        syn::Expr::Path(p) => Ok(p),
        _ => Err(syn::Error::new(ident.span(), error_message)),
    }
}

fn parse_ident_or_literal(expr: &Expr, span: Span, error_message: &str) -> Result<IdentOrLiteral> {
    match expr {
        Expr::Path(p) => {
            if p.path.segments.len() != 1 {
                return Err(syn::Error::new(span.clone(), error_message));
            }
            Ok(IdentOrLiteral::Path(p.clone()))
        }
        Expr::Lit(l) => match &l.lit {
            Lit::Str(_s) => Ok(IdentOrLiteral::Literal(l.clone())),
            _ => Err(syn::Error::new(span.clone(), error_message)),
        },
        _ => Err(syn::Error::new(span.clone(), error_message)),
    }
}

fn parse_list_expr(ident: &Ident, nv: &MetaNameValue, error_message: &str) -> Result<Vec<Expr>> {
    match &nv.value {
        syn::Expr::Array(a) => Ok(a.elems.iter().cloned().collect::<Vec<_>>()),
        _ => Err(syn::Error::new(ident.span(), error_message)),
    }
}

pub fn parse_module<'a>(ident: &'a Ident, nv: &'a MetaNameValue) -> Result<&'a ExprPath> {
    parse_expr_path(ident, nv, "module must be ident")
}

pub fn parse_list_display(ident: &Ident, nv: &MetaNameValue) -> Result<Vec<IdentOrLiteral>> {
    // parse_list_expr(ident, nv, "list_display must be array")
    match &nv.value {
        syn::Expr::Array(a) => a
            .elems
            .iter()
            .map(|e| {
                parse_ident_or_literal(
                    e,
                    ident.span(),
                    "list_display element must be Column or string literal",
                )
            })
            .collect::<Result<Vec<_>>>(),
        _ => Err(syn::Error::new(ident.span(), "list_display must be array")),
    }
}

pub fn parse_editable_fields(ident: &Ident, nv: &MetaNameValue) -> Result<Vec<Expr>> {
    parse_list_expr(ident, nv, "fields must be array")
}

pub fn parse_auto_complete(ident: &Ident, nv: &MetaNameValue) -> Result<Vec<Ident>> {
    parse_list_expr(ident, nv, "auto_complete must be array")?
        .iter()
        .map(|x| {
            parse_path_ident_from_expr(ident, x, "auto_complete element must be ident")
                .map(|x| x.clone())
        })
        .collect::<Result<Vec<_>>>()
}

pub fn parse_search_fields(ident: &Ident, nv: &MetaNameValue) -> Result<Vec<Expr>> {
    parse_list_expr(ident, nv, "search_fields must be array")
}

pub fn parse_format<'a>(ident: &'a Ident, nv: &'a MetaNameValue) -> Result<&'a Ident> {
    parse_path_ident(ident, nv, "format must be ident")
}

pub fn parse_initial_value<'a>(ident: &'a Ident, nv: &'a MetaNameValue) -> Result<&'a Ident> {
    parse_path_ident(ident, nv, "initial_value must be ident")
}

pub fn parse_ordering(ident: &Ident, nv: &MetaNameValue) -> Result<Vec<(Expr, Expr)>> {
    match &nv.value {
        syn::Expr::Array(a) => a
            .elems
            .iter()
            .map(|x| match x {
                syn::Expr::Tuple(t) => {
                    if t.elems.len() == 2 {
                        Ok((t.elems[0].clone(), t.elems[1].clone()))
                    } else {
                        Err(syn::Error::new(
                            ident.span(),
                            "ordering must be (Ident, Asc | Desc), each element must be two elements.",
                        ))
                    }
                }
                _ => Err(syn::Error::new(
                    ident.span(),
                    "ordering must be (Ident, Asc | Desc), each element must be tuple.",
                )),
            })
            .collect::<Result<Vec<_>>>(),
        _ => Err(syn::Error::new(ident.span(), "ordering must be array")),
    }
}

pub fn parse_form_fields(ident: &Ident, nv: &MetaNameValue) -> Result<Vec<Expr>> {
    match &nv.value {
        syn::Expr::Array(a) => Ok(a.elems.iter().map(|x| x.clone()).collect::<Vec<_>>()),
        _ => Err(syn::Error::new(
            ident.span(),
            "widgets must be array. [(Column, Widget), ..]",
        )),
    }
}
