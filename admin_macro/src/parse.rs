use syn::{Expr, ExprPath, Ident, MetaNameValue};

type Result<T> = std::result::Result<T, syn::Error>;

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

fn parse_list_expr(ident: &Ident, nv: &MetaNameValue, error_message: &str) -> Result<Vec<Expr>> {
    match &nv.value {
        syn::Expr::Array(a) => Ok(a.elems.iter().cloned().collect::<Vec<_>>()),
        _ => Err(syn::Error::new(ident.span(), error_message)),
    }
}

pub fn parse_module<'a>(ident: &'a Ident, nv: &'a MetaNameValue) -> Result<&'a ExprPath> {
    parse_expr_path(ident, nv, "module must be ident")
}

pub fn parse_list_display(ident: &Ident, nv: &MetaNameValue) -> Result<Vec<Expr>> {
    parse_list_expr(ident, nv, "list_display must be array")
}

pub fn parse_form_fields(ident: &Ident, nv: &MetaNameValue) -> Result<Vec<Expr>> {
    parse_list_expr(ident, nv, "fields must be array")
}

pub fn parse_auto_complete(ident: &Ident, nv: &MetaNameValue) -> Result<Vec<Expr>> {
    parse_list_expr(ident, nv, "auto_complete must be array")
}

pub fn parse_search_fields(ident: &Ident, nv: &MetaNameValue) -> Result<Vec<Expr>> {
    parse_list_expr(ident, nv, "search_fields must be array")
}

pub fn parse_format<'a>(ident: &'a Ident, nv: &'a MetaNameValue) -> Result<&'a Ident> {
    parse_path_ident(ident, nv, "format must be ident")
}

pub fn parse_default_value<'a>(ident: &'a Ident, nv: &'a MetaNameValue) -> Result<&'a Ident> {
    parse_path_ident(ident, nv, "default_value must be ident")
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
