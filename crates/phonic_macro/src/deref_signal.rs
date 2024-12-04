use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Expr, Generics, Ident, Path, PathSegment, PredicateType, Token, TraitBound, TraitBoundModifier,
    Type, TypeInfer, TypeParamBound, TypeReference, WhereClause, WherePredicate,
};

pub struct DerefSignalInput {
    _impl: Token![impl],
    generics: Generics,
    omitted: Vec<Ident>,
    _for: Token![for],
    signal: Path,
    target: Type,
    deref_self: TypeReference,
    deref: Expr,
    deref_where: Option<WhereClause>,
    deref_mut_self: Option<TypeReference>,
    deref_mut: Option<Expr>,
    deref_mut_where: Option<WhereClause>,
}

impl Parse for DerefSignalInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let _impl = input.parse()?;
        let mut generics = input.parse::<Generics>()?;

        input.parse::<TypeInfer>()?;
        let mut omitted = Vec::new();
        while input.peek(Token![+]) {
            input.parse::<Token![+]>()?;
            input.parse::<Token![!]>()?;
            let ident = input.parse()?;
            omitted.push(ident);
        }

        let _for = input.parse()?;
        let signal = input.parse()?;
        generics.where_clause = input.parse()?;

        let content;
        braced!(content in input);

        content.parse::<Token![type]>()?;
        let target_ident = content.parse::<Ident>()?;
        const TARGET_IDENT: &str = "Target";
        if target_ident != TARGET_IDENT {
            return Err(syn::Error::new(
                target_ident.span(),
                format!("expected ident {TARGET_IDENT}, found {target_ident}"),
            ));
        }

        content.parse::<Token![=]>()?;
        let target = content.parse()?;
        content.parse::<Token![;]>()?;

        let deref_self = content.parse()?;
        content.parse::<Token![->]>()?;
        let deref = content.parse()?;
        let deref_where = content.parse()?;
        content.parse::<Token![;]>()?;

        let (deref_mut_self, deref_mut, deref_mut_where) = if !content.is_empty() {
            let deref_mut_self = content.parse()?;
            content.parse::<Token![->]>()?;
            let deref_mut = content.parse()?;
            let deref_mut_where = content.parse()?;
            content.parse::<Token![;]>()?;

            (Some(deref_mut_self), Some(deref_mut), deref_mut_where)
        } else {
            (None, None, None)
        };

        Ok(DerefSignalInput {
            _impl,
            generics,
            omitted,
            _for,
            signal,
            target,
            deref_self,
            deref,
            deref_where,
            deref_mut_self,
            deref_mut,
            deref_mut_where,
        })
    }
}

pub fn generate_deref_signal_impl(input: DerefSignalInput) -> TokenStream {
    let DerefSignalInput {
        _impl,
        generics,
        omitted,
        _for,
        signal,
        target,
        deref_self,
        deref,
        deref_where,
        deref_mut_self,
        deref_mut,
        deref_mut_where,
    } = input;

    let traits = [
        "Signal",
        "IndexedSignal",
        "FiniteSignal",
        "SignalReader",
        "SignalWriter",
        "SignalSeeker",
    ];

    const HOST_CRATE: &str = "phonic_signal";
    let crate_root = if std::env::var("CARGO_PKG_NAME").unwrap().as_str() == HOST_CRATE {
        PathSegment::from(<Token![crate]>::default())
    } else {
        PathSegment::from(Ident::new(HOST_CRATE, Span::call_site()))
    };

    let omitted_iter = omitted.iter().map(|ident| ident.to_string());

    let qualified_traits = traits
        .iter()
        .filter(|_trait| !omitted_iter.clone().any(|ident| ident.as_str() == **_trait))
        .filter(|_trait| {
            !(matches!(**_trait, "SignalReader" | "SignalWriter" | "SignalSeeker")
                && deref_mut.is_none())
        })
        .map(|ident| Path {
            leading_colon: None,
            segments: Punctuated::from_iter([
                crate_root.clone(),
                PathSegment::from(Ident::new(ident, Span::call_site())),
            ]),
        });

    let where_clause = qualified_traits.clone().map(|path| {
        let mut clause = generics
            .where_clause
            .clone()
            .unwrap_or_else(|| WhereClause {
                where_token: <Token![where]>::default(),
                predicates: Punctuated::new(),
            });

        let trait_name = path.segments.last().unwrap().ident.to_string();

        if let Some(deref_where) = &deref_where {
            if matches!(
                trait_name.as_str(),
                "Signal" | "IndexedSignal" | "FiniteSignal"
            ) {
                clause.predicates.extend(deref_where.predicates.clone())
            }
        }

        if let Some(deref_mut_where) = &deref_mut_where {
            if matches!(
                trait_name.as_str(),
                "SignalReader" | "SignalWriter" | "SignalSeeker"
            ) {
                clause.predicates.extend(deref_mut_where.predicates.clone())
            }
        }

        let trait_bound = TraitBound {
            paren_token: None,
            modifier: TraitBoundModifier::None,
            lifetimes: None,
            path,
        };

        let predicate = PredicateType {
            lifetimes: None,
            bounded_ty: target.clone(),
            colon_token: <Token![:]>::default(),
            bounds: Punctuated::from_iter([TypeParamBound::Trait(trait_bound)]),
        };

        clause.predicates.push(WherePredicate::Type(predicate));

        clause
    });

    let impls = qualified_traits.clone().map(|path| {
        match path.segments.last().unwrap().ident.to_string().as_str() {
            "Signal" => quote! {
                type Sample = <#target as #path>::Sample;

                #[inline]
                fn spec(#deref_self) -> &#crate_root::SignalSpec {
                    <#target as #path>::spec(#deref)
                }
            },
            "IndexedSignal" => quote! {
                #[inline]
                fn pos(#deref_self) -> u64 {
                    <#target as #path>::pos(#deref)
                }
            },
            "FiniteSignal" => quote! {
                #[inline]
                fn len(#deref_self) -> u64 {
                    <#target as #path>::len(#deref)
                }
            },
            "SignalReader" => quote! {
                #[inline]
                fn read(#deref_mut_self, buf: &mut [Self::Sample]) -> #crate_root::PhonicResult<usize> {
                    <#target as #path>::read(#deref_mut, buf)
                }
            },
            "SignalWriter" => quote! {
                #[inline]
                fn write(#deref_mut_self, buf: &[Self::Sample]) -> #crate_root::PhonicResult<usize> {
                    <#target as #path>::write(#deref_mut, buf)
                }

                #[inline]
                fn flush(#deref_mut_self) -> #crate_root::PhonicResult<()> {
                    <#target as #path>::flush(#deref_mut)
                }
            },
            "SignalSeeker" => quote! {
                #[inline]
                fn seek(#deref_mut_self, offset: i64) -> #crate_root::PhonicResult<()> {
                    <#target as #path>::seek(#deref_mut, offset)
                }
            },
            _ => quote! {},
        }
    });

    let tokens = quote! {
        #(
            #_impl #generics #qualified_traits #_for #signal
            #where_clause
            {
                #impls
            }
        )*
    };

    tokens.into()
}
