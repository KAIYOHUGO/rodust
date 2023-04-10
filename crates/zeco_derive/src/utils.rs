use deluxe::{ParseAttributes, ParseMetaItem};
use syn::{parse_quote, Expr, Type};

#[derive(Debug, ParseAttributes, ParseMetaItem)]
#[deluxe(attributes(zeco, des, ser))]
pub struct DataArg {
    #[deluxe(default = parse_quote!(Box<dyn std::error::Error>))]
    pub error: Type,
    #[deluxe(default = parse_quote!(()))]
    pub arg: Type,
}

#[derive(Debug, ParseAttributes)]
#[deluxe(attributes(zeco, des, ser))]
pub struct DataEnumArg {
    #[deluxe(flatten)]
    pub enum_arg: EnumArg,
    #[deluxe(flatten)]
    pub data_arg: DataArg,
}

#[derive(Debug, ParseMetaItem)]
#[deluxe(attributes(zeco, des, ser))]
pub struct EnumArg {
    #[deluxe(default = parse_quote!(u8))]
    pub tag_repr: Type,
    pub tag_type: Option<Type>,
    #[deluxe(default = parse_quote!(()))]
    pub tag_arg: Type,
}

#[derive(Debug, Default, ParseAttributes)]
#[deluxe(default, attributes(zeco))]
pub struct StructFieldArg {
    pub arg: Option<Expr>,
    pub arg_des: Option<Expr>,
    pub arg_ser: Option<Expr>,

    #[deluxe(rename = if)]
    pub if_all: Option<Expr>,
    pub if_des: Option<Expr>,
    pub if_ser: Option<Expr>,

    pub default: Option<Expr>,
    pub default_des: Option<Expr>,
    pub default_ser: Option<Expr>,

    pub with: Option<Type>,
    pub with_des: Option<Type>,
    pub with_ser: Option<Type>,

    pub skip: Option<Expr>,
}

#[derive(Debug, Default, ParseAttributes)]
#[deluxe(default, attributes(zeco))]
pub struct EnumVariantArg {
    pub tag: Option<Expr>,
}

pub fn choice_1_or_err<T, E>(first: Option<T>, second: Option<T>, err: E) -> Result<Option<T>, E> {
    match (first, second) {
        (None, None) => Ok(None),
        (None, Some(e)) | (Some(e), None) => Ok(Some(e)),
        (Some(_), Some(_)) => Err(err),
    }
}
