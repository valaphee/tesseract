use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    parse_macro_input, parse_quote, spanned::Spanned, Data, DeriveInput, Field, Fields,
    GenericParam, Lifetime, LifetimeParam,
};

#[proc_macro_derive(Encode, attributes(using))]
pub fn derive_encode(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    fn field_encode(field: &Field, field_ref: TokenStream, references: bool) -> TokenStream {
        if let Some(using) = field
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("using"))
            .map(|attr| attr.parse_args::<Ident>().unwrap())
        {
            if references {
                quote_spanned! {
                    field.span() => unsafe { std::mem::transmute::<_, &#using>(#field_ref).encode(output) }
                }
            } else {
                quote_spanned! {
                    field.span() => #using(#field_ref).encode(output)
                }
            }
        } else {
            quote_spanned! {
                field.span() => #field_ref.encode(output)
            }
        }
    }

    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let body = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                if !fields.named.is_empty() {
                    let mut field_encodes = fields.named.iter().map(|field| {
                        field_encode(
                            field,
                            {
                                let field_name = field.ident.as_ref().unwrap();
                                quote! {
                                    self.#field_name
                                }
                            },
                            false,
                        )
                    });
                    let first_field_encode = field_encodes.next().unwrap();
                    quote! {
                        #first_field_encode #(?;#field_encodes)*
                    }
                } else {
                    quote! {
                        Ok(())
                    }
                }
            }
            Fields::Unit => quote! {
                Ok(())
            },
            _ => todo!(),
        },
        Data::Enum(data) => {
            let index_only = data
                .variants
                .iter()
                .all(|variant| matches!(variant.fields, Fields::Unit));
            let match_arms = data.variants.iter().enumerate().map(|(i, variant)| {
                let variant_index = i as i32;
                let variant_name = &variant.ident;
                match &variant.fields {
                    Fields::Named(fields) => {
                        if !fields.named.is_empty() {
                            let field_names = fields
                                .named
                                .iter()
                                .map(|field| field.ident.as_ref().unwrap());
                            let mut field_encodes = fields.named.iter().map(|field| field_encode(field, field.ident.as_ref().unwrap().into_token_stream(), true));
                            let first_field_encode = field_encodes.next().unwrap();
                            quote! {
                                Self::#variant_name { #(#field_names,)* } => {
                                    crate::types::VarI32(#variant_index).encode(output)?;
                                    #first_field_encode #(?;#field_encodes)*
                                }
                            }
                        } else {
                            quote! {
                                Self::#variant_name {} => crate::types::VarI32(#variant_index).encode(output),
                            }
                        }
                    }
                    Fields::Unnamed(fields) => {
                        if !fields.unnamed.is_empty() {
                            let field_names = (0..fields.unnamed.len()).map(|i| Ident::new(&format!("_{i}"), Span::call_site()));
                            let mut field_encodes = fields.unnamed.iter().enumerate().map(|(i, field)| field_encode(field, Ident::new(&format!("_{i}"), Span::call_site()).into_token_stream(), true));
                            let first_field_encode = field_encodes.next().unwrap();
                            quote! {
                                Self::#variant_name(#(#field_names,)*) => {
                                    crate::types::VarI32(#variant_index).encode(output)?;
                                    #first_field_encode #(?;#field_encodes)*
                                }
                            }
                        } else {
                            quote! {
                                Self::#variant_name() => crate::types::VarI32(#variant_index).encode(output),
                            }
                        }
                    }
                    Fields::Unit => {
                        if index_only {
                            quote! {
                                Self::#variant_name => #variant_index,
                            }
                        } else {
                            quote! {
                                Self::#variant_name => crate::types::VarI32(#variant_index).encode(output),
                            }
                        }
                    }
                }
            });
            if match_arms.len() == 0 {
                quote! {
                    unreachable!()
                }
            } else if index_only {
                quote! {
                    crate::types::VarI32(match self {
                        #(#match_arms)*
                    }).encode(output)
                }
            } else {
                quote! {
                    match self {
                        #(#match_arms)*
                    }
                }
            }
        }
        _ => todo!(),
    };

    proc_macro::TokenStream::from(quote! {
        impl #impl_generics Encode for #name #ty_generics
        #where_clause
        {
            fn encode(&self, output: &mut impl std::io::Write) -> crate::Result<()> {
                #body
            }
        }
    })
}

#[proc_macro_derive(Decode, attributes(using))]
pub fn derive_decode(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    fn field_decode(field: &Field) -> TokenStream {
        let field_name = field.ident.as_ref();
        if let Some(using) = field
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("using"))
            .map(|attr| attr.parse_args::<Ident>().unwrap())
        {
            if let Some(field_name) = field_name {
                quote_spanned! {
                    field.span() => #field_name: #using::decode(input)?.0
                }
            } else {
                quote_spanned! {
                    field.span() => #using::decode(input)?.0
                }
            }
        } else if let Some(field_name) = field_name {
            quote_spanned! {
                field.span() => #field_name: Decode::decode(input)?
            }
        } else {
            quote_spanned! {
                field.span() => Decode::decode(input)?
            }
        }
    }

    let mut input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let (_, ty_generics, _) = input.generics.split_for_impl();
    let ty_generics = ty_generics.to_token_stream();
    let lifetime = if let Some(lifetime) = input.generics.lifetimes().next() {
        lifetime.lifetime.clone()
    } else {
        let lifetime: Lifetime = parse_quote!('a);
        input
            .generics
            .params
            .push(GenericParam::Lifetime(LifetimeParam::new(lifetime.clone())));
        lifetime
    };
    let (impl_generics, _, where_clause) = input.generics.split_for_impl();

    let body = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                let field_decodes = fields.named.iter().map(field_decode);
                quote! {
                    Self {
                        #(#field_decodes,)*
                    }
                }
            }
            Fields::Unit => quote! {
                Self
            },
            _ => todo!(),
        },
        Data::Enum(data) => {
            let match_arms = data.variants.iter().enumerate().map(|(i, variant)| {
                let variant_index = i as i32;
                let variant_name = &variant.ident;
                match &variant.fields {
                    Fields::Named(fields) => {
                        let field_decodes = fields.named.iter().map(field_decode);
                        quote! {
                            #variant_index => Self::#variant_name {
                                #(#field_decodes,)*
                            }
                        }
                    }
                    Fields::Unnamed(fields) => {
                        let field_decodes = fields.unnamed.iter().map(field_decode);
                        quote! {
                            #variant_index => Self::#variant_name(#(#field_decodes,)*)
                        }
                    }
                    Fields::Unit => quote! {
                        #variant_index => Self::#variant_name
                    },
                }
            });
            quote! {
                match crate::types::VarI32::decode(input)?.0 {
                    #(#match_arms,)*
                    variant => return Err(crate::Error::UnknownVariant(variant))
                }
            }
        }
        _ => todo!(),
    };

    proc_macro::TokenStream::from(quote! {
        impl #impl_generics Decode<#lifetime> for #name #ty_generics
        #where_clause
        {
            fn decode(input: &mut &#lifetime [u8]) -> crate::Result<Self> {
                Ok(#body)
            }
        }
    })
}
