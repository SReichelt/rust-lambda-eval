use std::rc::Rc;

use proc_macro;
use proc_macro2::{token_stream::IntoIter, TokenStream, TokenTree};
use quote::{quote, quote_spanned};
use lambda_calculus::{context::Context, raw_expr::Param};

#[proc_macro]
pub fn raw_expr(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    parse_stream(input.into(), &Context::Root).into()
}

fn parse_stream(input: TokenStream, context: &Context) -> TokenStream {
    parse(&mut input.into_iter(), context)
}

fn parse(iter: &mut IntoIter, context: &Context) -> TokenStream {
    if let Some(mut result) = try_parse_one(iter, context) {
        while let Some(arg) = try_parse_one(iter, context) {
            let app = quote! {
                lambda_calculus::raw_expr::RawExpr::from(lambda_calculus::raw_expr::RawAppExpr {
                    fun: #result,
                    arg: #arg
                })
            };
            result = app.into();
        }
        result
    } else {
        quote! {
            compile_error!("expected expression")
        }
    }
}

fn try_parse_one(iter: &mut IntoIter, context: &Context) -> Option<TokenStream> {
    if let Some(token) = iter.next() {
        match &token {
            TokenTree::Group(group) => Some(parse_stream(group.stream(), context)),
            TokenTree::Ident(ident) => {
                let mut ident_str = ident.to_string();
                if let Some(ident_stripped) = ident_str.strip_prefix('λ') {
                    if ident_stripped.is_empty() {
                        if let Some(token) = iter.next() {
                            if let TokenTree::Ident(ident) = &token {
                                ident_str = ident.to_string();
                            } else {
                                return Some(quote_spanned! {
                                    token.span() =>
                                    compile_error!("expected identifier")
                                });
                            }
                        } else {
                            return Some(quote_spanned! {
                                token.span() =>
                                compile_error!("expected binder")
                            });
                        }
                    } else {
                        ident_str = ident_stripped.to_string();
                    }
                    let result = try_parse_binder_content(iter, &ident_str, context);
                    if result.is_some() {
                        result
                    } else {
                        Some(quote_spanned! {
                            token.span() =>
                            compile_error!("expected identifier")
                        })
                    }
                } else if let Some(idx) = context.get_var_index(&ident_str) {
                    Some(quote_spanned! {
                        token.span() =>
                        lambda_calculus::raw_expr::RawExpr::Var(#idx)
                    })
                } else {
                    Some(quote_spanned! {token.span() => (#token).clone()})
                }
            }
            _ => Some(quote_spanned! {
                token.span() =>
                compile_error!("expected expression")
            }),
        }
    } else {
        None
    }
}

fn try_parse_binder_content(
    iter: &mut IntoIter,
    name: &str,
    context: &Context,
) -> Option<TokenStream> {
    if let Some(token) = iter.next() {
        let param = Rc::new(Param { name: name.into() });
        let body_context = Context::Var {
            param: &param,
            parent: context,
        };
        match &token {
            TokenTree::Ident(ident) => {
                let ident_str = ident.to_string();
                if let Some(body) =
                    try_parse_binder_content(iter, &ident_str.to_string(), &body_context)
                {
                    // TODO: We should probably make use of spans here and at some other places.
                    Some(quote! {
                        lambda_calculus::raw_expr::RawExpr::from(lambda_calculus::raw_expr::RawLambdaExpr {
                            param: std::rc::Rc::new(Param { name: (#name).into() }),
                            body: #body
                        })
                    })
                } else {
                    Some(quote_spanned! {
                        token.span() =>
                        compile_error!("incomplete binder")
                    })
                }
            }
            TokenTree::Punct(punct) => {
                if punct.as_char() == '.' {
                    let body = parse(iter, &body_context);
                    Some(quote! {
                        lambda_calculus::raw_expr::RawExpr::from(lambda_calculus::raw_expr::RawLambdaExpr {
                            param: std::rc::Rc::new(Param { name: (#name).into() }),
                            body: #body
                        })
                    })
                } else {
                    Some(quote_spanned! {
                        token.span() =>
                        compile_error!("expected `.` or identifier")
                    })
                }
            }
            _ => Some(quote_spanned! {
                token.span() =>
                compile_error!("expected `.` or identifier")
            }),
        }
    } else {
        None
    }
}
