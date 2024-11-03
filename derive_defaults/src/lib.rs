use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Item, Meta};
use syn::punctuated::Punctuated;

#[cfg(feature = "serialization")]
static SERDE_DERIVES: &'static str = "serde::Serialize, serde::Deserialize,";

#[cfg(not(feature = "serialization"))]
static SERDE_DERIVES: &'static str = "";

#[proc_macro_attribute]
pub fn derive_defaults(meta: TokenStream, element: TokenStream) -> TokenStream {
    let derives = parse_macro_input!(meta with Punctuated::<Meta, syn::Token![,]>::parse_terminated).to_token_stream();
    let element = parse_macro_input!(element as Item);
    match element {
        Item::Struct(_) | Item::Enum(_) => {},
        _ => panic!("expected a struct or an enum"),
    };
    let element = element.into_token_stream();

    let optional_derives: proc_macro2::TokenStream = str::parse(SERDE_DERIVES).expect("failed to parse derive tokens");

    let tt = TokenStream::from(quote! {
        #[derive(Debug, #optional_derives #derives)]
        #element
    });

    println!("{}", tt.to_string());

    tt
}