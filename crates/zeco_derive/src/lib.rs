mod de;
mod utils;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Deserialize, attributes(zeco))]
pub fn deserialize(ts: TokenStream) -> TokenStream {
    let input = parse_macro_input!(ts as DeriveInput);
    let output = match de::deserialize(input) {
        Ok(output) => quote!(#output),
        Err(err) => err.into_compile_error(),
    };
    output.into()
}

#[cfg(test)]
mod tests {
    use syn::{parse_quote, Expr};

    use super::*;
    #[test]
    fn feature() {
        let a = 10usize;
        let b: Expr = parse_quote!(10);
        let e: Option<Expr> = Some(parse_quote!(#b + #a));
        dbg!(e);
    }

    #[test]
    fn deserialize() {
        let ast = parse_quote! {
            enum A {
                B,
                C,
                #[zeco(tag = 1..2)]
                D,
                E,
            }
            // struct A{
            //     #[zeco(with = WithU8)]
            //     a: &[u8]
            // }
        };
        let out = de::deserialize(ast);
        match out {
            Ok(t) => println!("{}", prettyplease::unparse(&parse_quote!(#t))),
            Err(e) => println!("Err : {}", e),
        }
    }
}
