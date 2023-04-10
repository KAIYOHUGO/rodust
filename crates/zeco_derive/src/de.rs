use deluxe::ParseAttributes;
use proc_macro2::{Ident, Literal, Span};
use quote::format_ident;
use syn::{
    parse_quote, parse_quote_spanned, spanned::Spanned, Arm, DataEnum, DeriveInput, Expr, Fields,
    ItemImpl, LifetimeDef, Path, Result, Stmt, Type,
};
use thiserror::Error;

use crate::utils::{
    choice_1_or_err, DataArg, DataEnumArg, EnumArg, EnumVariantArg, StructFieldArg,
};

pub fn deserialize(input: DeriveInput) -> Result<ItemImpl> {
    let (attr, (stmts, ret)) = match input.data {
        syn::Data::Struct(s) => {
            let attr = DataArg::parse_attributes(&input.attrs)?;
            let out = parse_fields(parse_quote!(Self), s.fields)?;
            (attr, out)
        }
        syn::Data::Enum(e) => {
            let attr = DataEnumArg::parse_attributes(&input.attrs)?;
            let out = parse_enum(e, attr.enum_arg)?;
            (attr.data_arg, out)
        }
        syn::Data::Union(u) => Err(Error::UnsupportedUnion.into_error(u.union_token.span))?,
    };

    let DataArg { error, arg } = attr;

    let (_, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut generics = input.generics.clone();
    let impl_generics = {
        let mut lifetime_de: LifetimeDef = parse_quote!('de);
        lifetime_de.bounds = input
            .generics
            .lifetimes()
            .map(|r| r.lifetime.clone())
            .collect();
        generics.params.push(parse_quote!(#lifetime_de));
        let (impl_generics, ..) = generics.split_for_impl();
        impl_generics
    };

    let name = input.ident;

    let output = parse_quote! {
        impl #impl_generics zeco::Deserialize<'de> for #name #ty_generics #where_clause {
            type Error = #error;
            type Arg<'arg> = #arg;

            fn deserialize<'arg>(buf: &'de [u8], offset: &mut usize, arg: Self::Arg<'arg>) -> Result<Self, Self::Error> {
                #(#stmts)*

                Ok(#ret)
            }
        }
    };
    Ok(output)
}

/// parsing stmt. return expr
fn parse_fields(type_path: Path, f: Fields) -> Result<(Vec<Stmt>, Expr)> {
    let mut stmts = vec![];
    let mut fields = vec![];

    // for speed's sake
    stmts.reserve(f.len());
    fields.reserve(f.len());

    let ret = match f {
        Fields::Named(named) => {
            for field in named.named {
                let attr = StructFieldArg::parse_attributes(&field)?;
                let span = field.span();
                let ty = field.ty;
                let name = field.ident.expect("never fail");
                let stmt = parse_field(name.clone(), ty, attr, span)?;
                stmts.push(stmt);
                fields.push(name);
            }
            parse_quote!(#type_path{#(#fields),*})
        }
        Fields::Unnamed(unnamed) => {
            for (i, field) in unnamed.unnamed.into_iter().enumerate() {
                let attr = StructFieldArg::parse_attributes(&field)?;
                let name = format_ident!("e{}", i);
                let span = field.span();
                let ty = field.ty;
                let stmt = parse_field(name.clone(), ty, attr, span)?;
                stmts.push(stmt);
                fields.push(name);
            }
            parse_quote!(#type_path(#(#fields),*))
        }
        Fields::Unit => parse_quote!(#type_path),
    };
    Ok((stmts, ret))
}

fn parse_field(name: Ident, ty: Type, attr: StructFieldArg, span: Span) -> Result<Stmt> {
    let arg: Expr = choice_1_or_err(
        attr.arg,
        attr.arg_des,
        Error::ConflictArg("arg").into_error(span),
    )?
    .unwrap_or(parse_quote!(Default::default()));

    let default: Expr = choice_1_or_err(
        attr.default,
        attr.default_des,
        Error::ConflictArg("default").into_error(span),
    )?
    .unwrap_or(parse_quote!(Default::default()));

    let if_arg = choice_1_or_err(
        attr.if_all,
        attr.if_des,
        Error::ConflictArg("if").into_error(span),
    )?;

    let with = choice_1_or_err(
        attr.with,
        attr.with_des,
        Error::ConflictArg("with").into_error(span),
    )?;
    let des_expr: Expr = match with {
        Some(with_ty) => {
            parse_quote! (<#with_ty as zeco::DeserializeWith<_>>::deserialize_with(buf, offset, #arg)?)
        }
        None => parse_quote!(zeco::Deserialize::deserialize(buf, offset, #arg)?),
    };

    let stmt = match if_arg {
        Some(e) => parse_quote! {
            let #name: #ty = if #e {
                #des_expr
            } else {
                #default
            };
        },
        None => parse_quote!(let #name: #ty = #des_expr;),
    };

    Ok(stmt)
}

fn parse_enum(
    e: DataEnum,
    EnumArg {
        tag_repr,
        tag_type,
        tag_arg,
    }: EnumArg,
) -> Result<(Vec<Stmt>, Expr)> {
    let mut arms: Vec<Arm> = vec![];
    let mut offset = 0usize;
    let mut prev_tag: Expr = parse_quote!(0);
    let mut const_stmts: Vec<Stmt> = vec![];

    for var in e.variants {
        let EnumVariantArg { mut tag } = ParseAttributes::parse_attributes(&var)?;
        let span = var.span();
        let name = var.ident;

        if let Some((_, discriminant)) = var.discriminant {
            if tag.is_none() {
                const_stmts.push(parse_quote!(const #name: #tag_repr = #discriminant;));
                tag = Some(parse_quote!(#name));
            }
            prev_tag = discriminant;
        };

        if tag.is_none() {
            let offset = Literal::usize_unsuffixed(offset);
            const_stmts.push(parse_quote!(const #name: #tag_repr = #prev_tag + #offset;));
            tag = Some(parse_quote!(#name));
        }

        let (stmt, result) = parse_fields(parse_quote!(Self::#name), var.fields)?;
        arms.push(parse_quote_spanned!(span=> #tag => {#(#stmt)* #result}));
        offset += 1;
    }

    let tag_type = tag_type.unwrap_or(tag_repr.clone());

    let stmts = parse_quote! {
        let tag: #tag_type = zeco::Deserialize::deserialize(buf, offset, #tag_arg)?;
        #[allow(non_upper_case_globals)]
        let ret = {
            #(#const_stmts)*
            match tag.into() {
                #(#arms)*
                _ => Err(zeco::des::Error::NoMatch)?,
            }
        };
    };
    let ret = parse_quote!(ret);
    Ok((stmts, ret))
}

#[derive(Debug, Error)]
pub enum Error<'s> {
    #[error("We did not support union type")]
    UnsupportedUnion,

    #[error("`{0}` cannot use with `{0}_des`")]
    ConflictArg(&'s str),
}

impl<'s> Error<'s> {
    pub fn into_error(self, span: Span) -> syn::Error {
        syn::Error::new(span, self)
    }
}
