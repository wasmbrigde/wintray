use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemStruct, parse_macro_input, parse_quote};

#[proc_macro_attribute]
pub fn wintray_template(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item = parse_macro_input!(input as ItemStruct);

    let mut has_askama = false;
    let mut template_attr_index = None;

    for (i, attr) in item.attrs.iter().enumerate() {
        if attr.path().is_ident("template") {
            template_attr_index = Some(i);
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("askama") {
                    has_askama = true;
                }
                Ok(())
            });
        }
    }

    let mod_name = format!("__wintray_askama_{}", item.ident);
    let mod_ident = syn::Ident::new(&mod_name, proc_macro2::Span::call_site());

    if let Some(index) = template_attr_index {
        if !has_askama {
            let attr = &mut item.attrs[index];
            if let syn::Meta::List(list) = &attr.meta {
                let old_tokens = &list.tokens;
                attr.meta = parse_quote! {
                    template(askama = self::#mod_ident, #old_tokens)
                };
            }
        }
    }

    let expanded = quote! {
        #[doc(hidden)]
        mod #mod_ident { pub use ::wintray::askama::*; }

        #[derive(::wintray::askama::Template)]
        #item
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn wintray_assets(_args: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemStruct);
    let ident = &item.ident;
    let vis = &item.vis;

    let mod_name = format!("__wintray_rust_embed_{}", ident);
    let mod_ident = syn::Ident::new(&mod_name, proc_macro2::Span::call_site());

    let expanded = quote! {
        #[doc(hidden)]
        mod #mod_ident {
            use ::wintray::rust_embed;

            #[derive(rust_embed::RustEmbed)]
            #item
        }
        #vis use #mod_ident::#ident;
    };

    TokenStream::from(expanded)
}
