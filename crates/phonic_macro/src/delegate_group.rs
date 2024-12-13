use crate::utils::CratePathVisitor;
use proc_macro2::{Delimiter, Group, Span, TokenStream, TokenTree};
use quote::{quote, ToTokens};
use std::collections::HashMap;
use syn::{
    braced,
    parse::{Parse, ParseStream, Parser},
    punctuated::Punctuated,
    spanned::Spanned,
    visit_mut::VisitMut,
    AttrStyle, Attribute, Ident, ItemTrait, Meta, MetaList, Path, PathArguments, PathSegment,
    PredicateType, Token, TraitItem, Type, TypePath, WherePredicate,
};

pub struct DelegateGroupInput {
    mod_path: Path,
    traits: Vec<ItemTrait>,
}

#[derive(Clone)]
pub struct TraitSignature {
    pub subgroups: Vec<Ident>,
    pub trait_token: Token![trait],
    pub path: Path,
    pub items: Vec<TraitItem>,
}

pub fn gen_delegate_group(mut input: DelegateGroupInput) -> syn::Result<TokenStream> {
    let mut subgroups = input.take_subgroup_attrs()?;
    let input_traits = input.traits.clone();
    let macro_ident = input.gen_macro_ident();

    let internal_macro_name = format!("_{}", macro_ident.clone());
    let internal_macro_ident = Ident::new(internal_macro_name.as_str(), macro_ident.span());

    input.expand_crate_paths();
    input.filter_bounded_items();

    let mut traits = input.into_trait_signatures();
    traits.iter_mut().for_each(|signature| {
        if let Some(subgroups) = subgroups.remove(signature.ident()) {
            signature.subgroups.extend(subgroups);
        }
    });

    Ok(quote! {
        #(#input_traits)*

        #[macro_export]
        macro_rules! #internal_macro_ident {
            ($($input:tt)*) => {
                ::phonic_macro::delegate_impl! {
                    #(#traits)*
                    $($input)*
                }
            };
        }

        pub use #internal_macro_ident as #macro_ident;
    })
}

impl DelegateGroupInput {
    fn take_subgroup_attrs(&mut self) -> syn::Result<HashMap<Ident, Vec<Ident>>> {
        let mut trait_subgroups = HashMap::new();

        for trait_ in &mut self.traits {
            let meta_list = trait_
                .attrs
                .iter()
                .enumerate()
                .find_map(|(i, attr)| match attr {
                    Attribute {
                        style: AttrStyle::Outer,
                        meta: Meta::List(list),
                        ..
                    } if list.path.segments.len() == 1
                        && list.path.segments.last().unwrap().ident == "subgroup" =>
                    {
                        Some((i, list))
                    }
                    _ => None,
                });

            let Some((attr_i, meta_list)) = meta_list else {
                continue;
            };

            let subgroup_list = Parser::parse2(
                Punctuated::<Ident, Token![,]>::parse_terminated,
                meta_list.tokens.clone(),
            )?;

            let subgroups = subgroup_list.into_iter().collect();
            trait_subgroups.insert(trait_.ident.clone(), subgroups);
            trait_.attrs.remove(attr_i);
        }

        Ok(trait_subgroups)
    }

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

    fn into_trait_signatures(self) -> Vec<TraitSignature> {
        let Self { mod_path, traits } = self;

        traits
            .into_iter()
            .map(|trait_| {
                let ItemTrait {
                    trait_token,
                    ident,
                    mut items,
                    ..
                } = trait_;

                let mut path = mod_path.clone();
                path.segments.push(PathSegment {
                    ident,
                    arguments: PathArguments::None,
                });

                items.iter_mut().for_each(|item| match item {
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
                });

                TraitSignature {
                    subgroups: Vec::new(),
                    trait_token,
                    path,
                    items,
                }
            })
            .collect()
    }
}

impl TraitSignature {
    pub fn ident(&self) -> &Ident {
        &self.path.segments.last().unwrap().ident
    }
}

impl Parse for DelegateGroupInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = Attribute::parse_inner(input)?;
        let mod_path = match attrs.as_slice() {
            [Attribute {
                meta:
                    Meta::List(MetaList {
                        path:
                            Path {
                                leading_colon: None,
                                segments,
                                ..
                            },
                        tokens,
                        ..
                    }),
                ..
            }] if segments.len() == 1 && segments.last().unwrap().ident == "mod_path" => {
                Parser::parse2(Path::parse_mod_style, tokens.clone())?
            }
            [] | [_] => {
                return Err(syn::Error::new(
                    input.span(),
                    "expected a mod_path attribute",
                ))
            }
            [..] => return Err(syn::Error::new(input.span(), "too many attributes")),
        };

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

impl Parse for TraitSignature {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = Attribute::parse_outer(input)?;
        let subgroups = match attrs.as_slice() {
            [] => Vec::new(),
            [Attribute {
                meta:
                    Meta::List(MetaList {
                        path:
                            Path {
                                leading_colon: None,
                                segments,
                            },
                        tokens,
                        ..
                    }),
                ..
            }] if segments.len() == 1 && segments.last().unwrap().ident == "subgroup" => {
                let subgroup_list = Parser::parse2(
                    Punctuated::<Ident, Token![,]>::parse_terminated,
                    tokens.clone(),
                )?;

                subgroup_list.into_iter().collect()
            }
            [attr] => return Err(syn::Error::new(attr.span(), "expected subgroup attribute")),
            [_, attr, ..] => return Err(syn::Error::new(attr.span(), "too many attributes")),
        };

        let trait_token = input.parse()?;
        let path = input.parse()?;

        let content;
        braced!(content in input);

        let mut items = Vec::new();
        while !content.is_empty() {
            items.push(content.parse()?);
        }

        Ok(Self {
            subgroups,
            trait_token,
            path,
            items,
        })
    }
}

impl ToTokens for TraitSignature {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if !self.subgroups.is_empty() {
            let subgroups = &self.subgroups;
            let attr = quote! { #[subgroup(#(#subgroups),*)] };
            attr.to_tokens(tokens);
        }

        self.trait_token.to_tokens(tokens);
        self.path.to_tokens(tokens);

        let mut item_tokens = TokenStream::new();
        self.items
            .iter()
            .for_each(|item| item.to_tokens(&mut item_tokens));

        let items = TokenTree::Group(Group::new(Delimiter::Brace, item_tokens));
        items.to_tokens(tokens);
    }
}
