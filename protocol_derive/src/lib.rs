use proc_macro2::{Ident, Span};
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, Data, DeriveInput, Fields};

#[proc_macro_derive(Encode)]
pub fn derive_encode(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let input_name = input.ident;
    let body = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                let mut field_encodes = fields.named.iter().map(|field| {
                    let field_name = &field.ident;
                    quote_spanned! {
                        field.span() => self.#field_name.encode(output)
                    }
                });
                let first_field_encode = field_encodes.next().unwrap();
                quote! {
                    #first_field_encode #(?;#field_encodes)*
                }
            },
            Fields::Unit => quote! {
                Ok(())
            },
            _ => unreachable!(),
        }
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
                        let field_names = fields
                            .named
                            .iter()
                            .map(|field| field.ident.as_ref().unwrap());
                        let mut field_encodes = fields.named.iter().map(|field| {
                            let field_name = &field.ident;
                            quote_spanned! {
                                field.span() => #field_name.encode(output)
                            }
                        });
                        let first_field_encode = field_encodes.next().unwrap();
                        quote! {
                            Self::#variant_name { #(#field_names,)* } => {
                                crate::VarInt(#variant_index).encode(output)?;
                                #first_field_encode #(?;#field_encodes)*
                            }
                        }
                    }
                    Fields::Unnamed(fields) => {
                        let field_names = (0..fields.unnamed.len()).map(|i| Ident::new(&format!("_{}", i), Span::call_site()));
                        let mut field_encodes = fields.unnamed.iter().enumerate().map(|(i, field)| {
                            let field_name = Ident::new(&format!("_{}", i), Span::call_site());
                            quote_spanned! {
                                field.span() => #field_name.encode(output)
                            }
                        });
                        let first_field_encode = field_encodes.next().unwrap();
                        quote! {
                            Self::#variant_name(#(#field_names,)*) => {
                                crate::VarInt(#variant_index).encode(output)?;
                                #first_field_encode #(?;#field_encodes)*
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
                                Self::#variant_name => crate::VarInt(#variant_index).encode(output),
                            }
                        }
                    }
                }
            });
            if index_only {
                quote! {
                    crate::VarInt(match self {
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
        _ => unreachable!(),
    };
    proc_macro::TokenStream::from(quote! {
        impl Encode for #input_name {
            fn encode<W: std::io::Write>(&self, output: &mut W) -> anyhow::Result<()> {
                #body
            }
        }
    })
}

#[proc_macro_derive(Decode)]
pub fn derive_decode(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let input_name = input.ident;
    let body = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                let field_decodes = fields.named.iter().map(|field| {
                    let field_name = &field.ident;
                        quote_spanned! {
                        field.span() => #field_name: Decode::decode(input)?
                    }
                });
                quote! {
                    Self {
                        #(#field_decodes,)*
                    }
                }
            },
            Fields::Unit => quote! {
                Self
            },
            _ => unreachable!(),
        }
        Data::Enum(data) => {
            let match_arms = data.variants.iter().enumerate().map(|(i, variant)| {
                let variant_index = i as i32;
                let variant_name = &variant.ident;
                match &variant.fields {
                    Fields::Named(fields) => {
                        let field_decodes = fields.named.iter().map(|field| {
                            let field_name = &field.ident;
                            quote_spanned! {
                                field.span() => #field_name: Decode::decode(input)?
                            }
                        });
                        quote! {
                            #variant_index => Self::#variant_name {
                                #(#field_decodes,)*
                            }
                        }
                    }
                    Fields::Unnamed(fields) => {
                        let field_decodes = fields.unnamed.iter().map(|field| {
                            quote_spanned! {
                                field.span() => Decode::decode(input)?
                            }
                        });
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
                match crate::VarInt::decode(input)?.0 {
                    #(#match_arms,)*
                     _ => unreachable!()
                }
            }
        }
        _ => unreachable!(),
    };
    proc_macro::TokenStream::from(quote! {
        impl Decode for #input_name {
            fn decode<R: std::io::Read>(input: &mut R) -> anyhow::Result<Self> {
                Ok(#body)
            }
        }
    })
}
