use proc_macro::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{Ident, LitBool, Token, parse::Parse};

struct BoardMacroInput {
    name: Ident,
    _comma: Token![,],
    reversed: LitBool,
}
impl Parse for BoardMacroInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(BoardMacroInput {
            name: input.parse()?,
            _comma: input.parse()?,
            reversed: input.parse()?,
        })
    }
}
#[proc_macro]
pub fn create_board_enum(items: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(items as BoardMacroInput);
    let mut variants = Vec::new();
    let mut letters = ('A'..='H').cycle();
    let is_reversed: bool = input.reversed.value();
    let name = input.name;
    for row in 0..8u8 {
        for _ in 0..8u8 {
            let letter = letters.next().expect("it works");
            let var_name = format_ident!("{}{}", letter, (row + 1));
            variants.push(var_name);
        }
    }
    if is_reversed {
        variants.reverse();
    }
    quote! {
        #[derive(Debug, Clone, PartialEq)]
        pub struct BoardError(pub String);

        impl std::fmt::Display for BoardError {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl std::error::Error for BoardError {}

        #[repr(u8)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum #name {
            #(#variants),*
        }
        impl #name {
            pub const fn from_u8_unchecked(v: u8) -> Self {
                if v >= 64 { panic!("Square index out of bounds"); }
                unsafe { std::mem::transmute(v) }
            }
        }
        impl TryFrom<u8> for #name {
            type Error = BoardError;
            fn try_from(v: u8) -> Result<Self, Self::Error> {
                if  v < 64 {
                    unsafe { Ok(std::mem::transmute::<u8, #name>(v)) }
                } else {
                    Err(BoardError(format!("Value {} is out of range for {}", v, stringify!(#name))))
                }
            }
        }
    }
    .into()
}

/*
* pub enum Square {
* A1, A2 ... H8
  }
*
*
*
*/
