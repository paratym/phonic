use crate::utils::CratePathVisitor;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    visit_mut::VisitMut,
    Attribute, Ident, ItemTrait, Meta, MetaList, Path, PathSegment, PredicateType, Token,
    TraitItem, Type, TypePath, WherePredicate,
};

pub struct DelegateGroupInput {
    mod_path: Path,
    traits: Vec<ItemTrait>,
}

pub fn gen_delegate_group(mut input: DelegateGroupInput) -> syn::Result<TokenStream> {
    let mut input_traits = input.traits.clone();
    input_traits.iter_mut().for_each(remove_attrs);

    let macro_ident = input.gen_macro_ident();
    let internal_macro_name = format!("_{}", macro_ident.clone());
    let internal_macro_ident = Ident::new(internal_macro_name.as_str(), macro_ident.span());

    input.filter_bounded_items();
    input.remove_default_impls();
    input.expand_crate_paths();

    let DelegateGroupInput { mod_path, traits } = input;

    Ok(quote! {
        #(#input_traits)*

        #[macro_export]
        macro_rules! #internal_macro_ident {
            ($($input:tt)*) => {
                ::phonic_macro::delegate_impl! {
                    mod as #mod_path;
                    #(#traits)*
                    $($input)*
                }
            };
        }

        pub use #internal_macro_ident as #macro_ident;
    })
}

impl DelegateGroupInput {
    fn gen_macro_ident(&self) -> Ident {
        let base_trait = self.traits.first().unwrap();
        let macro_ident_str = format!("delegate_{}", base_trait.ident).to_lowercase();

        Ident::new(&macro_ident_str, Span::call_site())
    }

    fn expand_crate_paths(&mut self) {
        let mut visitor = CratePathVisitor::expand();

        visitor.visit_path_mut(&mut self.mod_path);
        self.traits
            .iter_mut()
            .for_each(|trait_| visitor.visit_item_trait_mut(trait_));
    }

    fn filter_bounded_items(&mut self) {
        self.traits.iter_mut().for_each(|trait_| {
            trait_.items.retain(|item| match &item {
                TraitItem::Fn(fn_item) => {
                    fn_item
                        .sig
                        .generics
                        .where_clause
                        .as_ref()
                        .is_none_or(|clause| {
                            !clause.predicates.iter().any(|predicate| {
                                matches!(predicate, WherePredicate::Type(PredicateType {
                                    bounded_ty: Type::Path(TypePath { path, .. }),
                                    ..
                                }) if path.segments.first().is_some_and(
                                    |PathSegment { ident, .. }| {
                                        *ident == Ident::from(<Token![Self]>::default())
                                    },
                                ))
                            })
                        })
                }
                _ => true,
            })
        });
    }

    fn remove_default_impls(&mut self) {
        self.traits.iter_mut().for_each(|trait_| {
            trait_.items.iter_mut().for_each(|item| match item {
                TraitItem::Type(type_) => {
                    type_.attrs.clear();
                    type_.colon_token = None;
                    type_.bounds.clear();
                    type_.default = None;
                }
                TraitItem::Const(const_) => {
                    const_.attrs.clear();
                    const_.default = None;
                }
                TraitItem::Fn(fn_) => {
                    fn_.attrs.clear();
                    fn_.default = None;
                }
                _ => {}
            })
        });
    }
}

impl Parse for DelegateGroupInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        <Token![mod]>::parse(input)?;
        <Token![as]>::parse(input)?;
        let mod_path = input.parse()?;
        <Token![;]>::parse(input)?;

        let mut traits = Vec::new();
        while !input.is_empty() {
            traits.push(input.parse()?)
        }

        if traits.is_empty() {
            return Err(syn::Error::new(Span::call_site(), "expected a trait"));
        }

        Ok(Self { mod_path, traits })
    }
}

fn remove_attrs(trait_: &mut ItemTrait) {
    let attrs = trait_.attrs.iter().filter(|attr| match attr {
        Attribute {
            meta: Meta::List(MetaList { path, .. }),
            ..
        } if path
            .segments
            .last()
            .is_some_and(|seg| seg.ident == "subgroup") =>
        {
            false
        }
        Attribute {
            meta: Meta::List(MetaList { path, .. }),
            ..
        } if path.segments.last().is_some_and(|seg| seg.ident == "rcv") => false,
        _ => true,
    });

    trait_.attrs = attrs.cloned().collect();
}
