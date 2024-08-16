extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Ident, Token};
use syn::parse::{Parse, ParseStream};

struct MacroInput {
	type_names: syn::punctuated::Punctuated<Ident, Token![,]>,
}

impl Parse for MacroInput {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let type_names = syn::punctuated::Punctuated::parse_terminated(input)?;
		Ok(MacroInput { type_names })
	}
}

#[proc_macro]
pub fn generate_unique_types(input: TokenStream) -> TokenStream {
	let MacroInput { type_names } = parse_macro_input!(input as MacroInput);
	
	let mut generated_types = vec![];
	
	for (i, with_type) in type_names.iter().enumerate() {
		let without_types = type_names.iter().enumerate().filter_map(|(j, f)| {
			if i != j {
				Some(f)
			} else {
				None
			}
		});
		
		let without_types_tokens = without_types.map(|f| {
			quote! { Without<#f> }
		});
		
		let unique_type_name = Ident::new(&format!("Unique{}", with_type), with_type.span());
		
		generated_types.push(quote! {
			pub type #unique_type_name = (
				With<#with_type>,
				#(#without_types_tokens),*
			);
		});
	}
	
	let expanded = quote! {
		#(#generated_types)*
	};
	
	TokenStream::from(expanded)
}