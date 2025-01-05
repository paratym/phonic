use crate::utils::CratePathVisitor;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    braced,
    parse::{Parse, ParseStream, Parser},
    punctuated::Punctuated,
    spanned::Spanned,
    token::{Brace, Impl, Paren},
    visit_mut::VisitMut,
    Attribute, Block, Expr, ExprCall, ExprPath, FnArg, Generics, Ident, ImplItem, ImplItemConst,
    ImplItemFn, ImplItemType, ItemImpl, ItemTrait, Local, LocalInit, Meta, MetaList, Pat, PatIdent,
    Path, PathSegment, PredicateType, QSelf, Receiver, Stmt, Token, TraitBound, TraitBoundModifier,
    TraitItem, TraitItemFn, Type, TypeParamBound, TypePath, Visibility, WhereClause,
    WherePredicate,
};

pub struct DelegateImplInput {
    mod_path: Path,
    traits: Vec<ItemTrait>,
    block: DelegateBlock,
}

struct DelegateBlock {
    impl_token: Token![impl],
    generics: Generics,
    selector: TraitSelector,
    for_token: Token![for],
    self_ty: Type,

    delegate_ty: Type,
    delegate_self: Option<DelegateBranch>,
    delegate_ref: Option<DelegateBranch>,
    delegate_mut: Option<DelegateBranch>,
}

enum TraitSelector {
    Explicit { included: Vec<Ident> },
    Wildcard { omitted: Vec<Ident> },
}

#[derive(Clone)]
struct DelegateBranch {
    rcv: Receiver,
    expr: Expr,
    where_clause: Option<WhereClause>,
}

pub fn gen_delegate_impl(mut input: DelegateImplInput) -> syn::Result<TokenStream> {
    input.inline_crate_paths();
    input.filter_subgroups()?;
    let impl_items = input.into_impl_items()?;

    Ok(quote! { #(#impl_items)* })
}

impl DelegateImplInput {
    fn inline_crate_paths(&mut self) {
        let mut visitor = CratePathVisitor::inline_strict();

        visitor.visit_path_mut(&mut self.mod_path);
        self.traits
            .iter_mut()
            .for_each(|trait_| visitor.visit_item_trait_mut(trait_));
    }

    fn filter_subgroups(&mut self) -> syn::Result<()> {
        let filtered = self
            .traits
            .iter()
            .map(|trait_| {
                self.block
                    .selector
                    .includes(trait_)
                    .map(|included| (included, trait_))
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .filter(|(included, _)| *included)
            .map(|(_, trait_)| trait_)
            .cloned()
            .collect();

        self.traits = filtered;

        Ok(())
    }

    fn into_impl_items(self) -> syn::Result<Vec<ItemImpl>> {
        let Self {
            mod_path,
            traits,
            block,
        } = self;

        traits
            .into_iter()
            .map(|signature| block.gen_trait_impl(&mod_path, signature))
            .collect()
    }
}

impl DelegateBlock {
    fn gen_trait_impl(&self, mod_path: &Path, signature: ItemTrait) -> syn::Result<ItemImpl> {
        let ItemTrait { ident, items, .. } = signature;

        let mut generics = self.generics.clone();
        let where_clause = generics.where_clause.get_or_insert(WhereClause {
            where_token: Default::default(),
            predicates: Punctuated::new(),
        });

        let rcv_delegates = items
            .iter()
            .fold([None, None, None], |mut predicates, item| {
                if let TraitItem::Fn(fn_item) = item {
                    match fn_item.sig.receiver() {
                        Some(Receiver {
                            reference: None, ..
                        }) if predicates[0].is_none() => predicates[0] = self.delegate_self.clone(),
                        Some(Receiver {
                            reference: Some(_),
                            mutability: None,
                            ..
                        }) if predicates[1].is_none() => predicates[1] = self.delegate_ref.clone(),
                        Some(Receiver {
                            reference: Some(_),
                            mutability: Some(_),
                            ..
                        }) if predicates[2].is_none() => predicates[2] = self.delegate_mut.clone(),
                        _ => {}
                    }
                }

                predicates
            });

        let rcv_predicates = rcv_delegates
            .into_iter()
            .flatten()
            .filter_map(|branch| branch.where_clause)
            .flat_map(|clause| clause.predicates.into_iter());

        where_clause.predicates.extend(rcv_predicates);

        // if !supertraits.is_empty() {
        //     let super_predicates = WherePredicate::Type(PredicateType {
        //         lifetimes: None,
        //         bounded_ty: self.self_ty.clone(),
        //         colon_token: Default::default(),
        //         bounds: supertraits,
        //     });
        //
        //     where_clause.predicates.push(super_predicates);
        // }

        let mut trait_path = mod_path.clone();
        trait_path.segments.push(ident.into());

        let delegate_predicate = WherePredicate::Type(PredicateType {
            lifetimes: None,
            bounded_ty: self.delegate_ty.clone(),
            colon_token: Default::default(),
            bounds: Punctuated::from_iter([TypeParamBound::Trait(TraitBound {
                paren_token: None,
                modifier: TraitBoundModifier::None,
                lifetimes: None,
                path: trait_path.clone(),
            })]),
        });

        where_clause.predicates.push(delegate_predicate);

        let impl_items = items
            .into_iter()
            .map(|item| self.gen_trait_item_impl(&trait_path, item))
            .collect::<Result<_, _>>()?;

        Ok(ItemImpl {
            attrs: Vec::new(),
            defaultness: None,
            unsafety: None,
            impl_token: self.impl_token,
            generics,
            trait_: Some((None, trait_path, self.for_token)),
            self_ty: Box::new(self.self_ty.clone()),
            brace_token: Brace::default(),
            items: impl_items,
        })
    }

    fn gen_trait_item_impl(&self, path: &Path, item: TraitItem) -> syn::Result<ImplItem> {
        let item_ident = match &item {
            TraitItem::Type(item) => item.ident.clone(),
            TraitItem::Const(item) => item.ident.clone(),
            TraitItem::Fn(item) => item.sig.ident.clone(),
            item => return Err(syn::Error::new(item.span(), "unsupported")),
        };

        let mut item_path = path.clone();
        item_path
            .segments
            .push(PathSegment::from(item_ident.clone()));

        let qself = QSelf {
            lt_token: <Token![<]>::default(),
            ty: Box::new(self.delegate_ty.clone()),
            as_token: Some(<Token![as]>::default()),
            gt_token: <Token![>]>::default(),
            position: item_path.segments.len() - 1,
        };

        match item {
            TraitItem::Type(item) => Ok(ImplItem::Type(ImplItemType {
                attrs: item.attrs.clone(),
                vis: Visibility::Inherited,
                defaultness: None,
                type_token: item.type_token,
                ident: item.ident.clone(),
                generics: item.generics.clone(),
                eq_token: <Token![=]>::default(),
                ty: Type::Path(TypePath {
                    qself: Some(qself),
                    path: item_path,
                }),
                semi_token: item.semi_token,
            })),
            TraitItem::Const(item) => Ok(ImplItem::Const(ImplItemConst {
                attrs: item.attrs.clone(),
                vis: Visibility::Inherited,
                defaultness: None,
                const_token: <Token![const]>::default(),
                ident: item.ident.clone(),
                generics: item.generics.clone(),
                colon_token: item.colon_token,
                ty: item.ty.clone(),
                eq_token: <Token![=]>::default(),
                expr: Expr::Path(ExprPath {
                    attrs: Vec::new(),
                    qself: Some(qself),
                    path: item_path,
                }),
                semi_token: item.semi_token,
            })),
            TraitItem::Fn(item) => {
                let TraitItemFn { attrs, mut sig, .. } = item;

                let rcv = sig
                    .receiver()
                    .map(|rcv| match (&rcv.reference, &rcv.mutability) {
                        (None, _) => self.delegate_self.clone(),
                        (Some(_), None) => self.delegate_ref.clone(),
                        (Some(_), Some(_)) => self.delegate_mut.clone(),
                    })
                    .map(|rcv| {
                        let msg = format!(
                            "trait method {} required an undefined delegation",
                            &item_ident
                        );

                        rcv.ok_or(syn::Error::new(
                            sig.receiver().unwrap().span(),
                            msg.as_str(),
                        ))
                    })
                    .transpose()?;

                sig.inputs
                    .iter_mut()
                    .enumerate()
                    .for_each(|(i, param)| match param {
                        FnArg::Receiver(receiver) => {
                            let DelegateBranch { rcv, .. } = rcv.as_ref().unwrap();
                            *receiver = rcv.clone();
                        }
                        FnArg::Typed(pattern) => {
                            pattern.pat = Box::new(Pat::Ident(PatIdent {
                                attrs: Vec::new(),
                                by_ref: None,
                                mutability: None,
                                ident: Ident::new(format!("_{i}").as_str(), pattern.pat.span()),
                                subpat: None,
                            }))
                        }
                    });

                let rcv_delegate = rcv.map(|branch| {
                    Stmt::Local(Local {
                        attrs: Vec::new(),
                        let_token: <Token![let]>::default(),
                        pat: Pat::Ident(PatIdent {
                            attrs: Vec::new(),
                            by_ref: None,
                            mutability: None,
                            ident: Ident::new("delegate", branch.rcv.span()),
                            subpat: None,
                        }),
                        init: Some(LocalInit {
                            eq_token: <Token![=]>::default(),
                            expr: Box::new(branch.expr),
                            diverge: None,
                        }),
                        semi_token: <Token![;]>::default(),
                    })
                });

                let args = sig.inputs.iter().enumerate().map(|(i, param)| {
                    let ident = match param {
                        FnArg::Receiver(rcv_param) => Ident::new("delegate", rcv_param.span()),
                        FnArg::Typed(type_param) => {
                            Ident::new(format!("_{i}").as_str(), type_param.pat.span())
                        }
                    };

                    Expr::Path(ExprPath {
                        attrs: Vec::new(),
                        qself: None,
                        path: Path::from(ident),
                    })
                });

                let call = Stmt::Expr(
                    Expr::Call(ExprCall {
                        attrs: Vec::new(),
                        func: Box::new(Expr::Path(ExprPath {
                            attrs: Vec::new(),
                            qself: Some(qself),
                            path: item_path,
                        })),
                        paren_token: Paren::default(),
                        args: args.collect(),
                    }),
                    None,
                );

                let _stmts = [rcv_delegate, Some(call)];
                let stmts = _stmts.into_iter().flatten().collect();

                Ok(ImplItem::Fn(ImplItemFn {
                    attrs,
                    vis: Visibility::Inherited,
                    defaultness: None,
                    sig,
                    block: Block {
                        brace_token: Default::default(),
                        stmts,
                    },
                }))
            }
            _ => unreachable!(),
        }
    }
}

impl TraitSelector {
    fn includes(&self, trait_: &ItemTrait) -> syn::Result<bool> {
        let (idents, explicit) = match self {
            Self::Explicit { included } => (included, true),
            Self::Wildcard { omitted } => (omitted, false),
        };

        let subgroups = match trait_.attrs.as_slice() {
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
            // [_, attr, ..] => return Err(syn::Error::new(attr.span(), "too many attributes")),
            _ => Vec::new(),
        };

        let matches = idents.iter().any(|ident| {
            *ident == trait_.ident || subgroups.iter().any(|subgroup| ident == subgroup)
        });

        Ok(matches == explicit)
    }
}

impl Parse for DelegateImplInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        <Token![mod]>::parse(input)?;
        <Token![as]>::parse(input)?;
        let mod_path = input.parse()?;
        <Token![;]>::parse(input)?;

        let mut traits = Vec::new();
        while !input.peek(Token![impl]) {
            traits.push(input.parse()?);
        }

        let block = input.parse()?;

        Ok(Self {
            mod_path,
            traits,
            block,
        })
    }
}

impl Parse for DelegateBlock {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let impl_token = input.parse()?;

        let generics = input.parse()?;
        let selector = input.parse()?;
        let for_token = input.parse()?;
        let self_ty = input.parse()?;

        let content;
        braced!(content in input);

        <Token![Self]>::parse(&content)?;
        <Token![as]>::parse(&content)?;
        let delegate_ty = content.parse()?;
        <Token![;]>::parse(&content)?;

        let mut branches = Vec::new();
        while !content.is_empty() {
            branches.push(DelegateBranch::parse(&content)?);
        }

        let delegate_self = branches
            .iter()
            .enumerate()
            .find_map(|(i, branch)| match branch.rcv {
                Receiver {
                    reference: None, ..
                } => Some(i),
                _ => None,
            })
            .map(|i| branches.remove(i));

        let delegate_ref = branches
            .iter()
            .enumerate()
            .find_map(|(i, branch)| match branch.rcv {
                Receiver {
                    reference: Some(_),
                    mutability: None,
                    ..
                } => Some(i),
                _ => None,
            })
            .map(|i| branches.remove(i));

        let delegate_mut = branches
            .iter()
            .enumerate()
            .find_map(|(i, branch)| match branch.rcv {
                Receiver {
                    reference: Some(_),
                    mutability: Some(_),
                    ..
                } => Some(i),
                _ => None,
            })
            .map(|i| branches.remove(i));

        if let Some(branch) = branches.first() {
            return Err(syn::Error::new(
                branch.rcv.span(),
                "duplicate receiver delegation",
            ));
        }

        Ok(Self {
            impl_token,
            generics,
            selector,
            for_token,
            self_ty,

            delegate_ty,
            delegate_self,
            delegate_ref,
            delegate_mut,
        })
    }
}

impl Parse for TraitSelector {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Token![*]) {
            <Token![*]>::parse(input)?;
            let mut omitted = Vec::new();
            while input.peek(Token![+]) {
                <Token![+]>::parse(input)?;
                <Token![!]>::parse(input)?;
                omitted.push(input.parse()?);
            }

            return Ok(Self::Wildcard { omitted });
        }

        let mut included = Vec::new();
        loop {
            included.push(input.parse()?);

            if input.peek(Token![+]) {
                <Token![+]>::parse(input)?;
            } else {
                break;
            }
        }

        Ok(Self::Explicit { included })
    }
}

impl Parse for DelegateBranch {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let rcv = input.parse()?;
        <Token![=>]>::parse(input)?;
        let expr = input.parse()?;
        let where_clause = input.parse()?;
        <Token![;]>::parse(input)?;

        Ok(Self {
            rcv,
            expr,
            where_clause,
        })
    }
}
