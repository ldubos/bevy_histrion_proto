mod attributes;

use attributes::PrototypeAttributes;
use proc_macro::TokenStream;
use proc_macro2_diagnostics::*;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, parse_macro_input, spanned::Spanned};

#[proc_macro_derive(Prototype, attributes(proto))]
pub fn prototype_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let Data::Struct(struct_data) = &input.data else {
        return input
            .span()
            .error("Prototype can only be derived for structs")
            .emit_as_expr_tokens()
            .into();
    };

    if struct_data.fields.is_empty() {
        return input
            .span()
            .error("Prototype cannot be derived for empty structs")
            .emit_as_expr_tokens()
            .into();
    }

    if !matches!(struct_data.fields, Fields::Named(_)) {
        return input
            .span()
            .error("Prototype can only be derived for structs with named fields")
            .emit_as_expr_tokens()
            .into();
    }

    let discriminant = {
        let mut attrs = PrototypeAttributes::default();

        for attr in &input.attrs {
            if attr.path().is_ident("proto") {
                if let Err(err) = attributes::parse_attributes(attr, false, &mut attrs) {
                    return err.emit_as_expr_tokens().into();
                }
            }
        }

        attrs.discriminant
    };

    if discriminant.is_none() {
        return input
            .span()
            .error("discriminant attribute is required")
            .emit_as_expr_tokens()
            .into();
    }

    let ident = &input.ident;
    let raw_prototype_ident = format_ident!("{}Raw", ident);
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut id = None;
    let mut fields = Vec::new();
    let mut from_raw_body = quote!();
    let mut default_functions = quote!();

    for field in &struct_data.fields {
        let mut attrs = PrototypeAttributes::default();

        for attr in &field.attrs {
            if attr.path().is_ident("proto") {
                if let Err(err) = attributes::parse_attributes(attr, true, &mut attrs) {
                    return err.emit_as_expr_tokens().into();
                }
            }
        }

        let field_ident = field.ident.clone().unwrap();
        let field_type = &field.ty;

        if attrs.is_id {
            if id.is_some() {
                return field
                    .span()
                    .error("only one field can be marked as id")
                    .emit_as_expr_tokens()
                    .into();
            }

            match field_type {
                syn::Type::Path(type_path) => {
                    let field_type = type_path.path.segments.last().unwrap();

                    match field_type.ident.to_string().as_str() {
                        "Id" => {
                            id = Some(quote! {
                                self.#field_ident
                            });
                        }
                        "NamedId" => {
                            id = Some(quote! {
                                self.#field_ident.id()
                            });
                        }
                        _ => {
                            return field_type
                                .span()
                                .error("the id field must have the type `Id<T>` or `NamedId<T>`")
                                .emit_as_expr_tokens()
                                .into();
                        }
                    }
                }
                _ => {
                    return field_type
                        .span()
                        .error("the id field must have the type `Id<T>` or `NamedId<T>`")
                        .emit_as_expr_tokens()
                        .into();
                }
            }
        }

        let default_attr = if let Some(default_func) = attrs.default_func {
            quote! {
                #[serde(default = #default_func)]
            }
        } else if let Some(default_expr) = attrs.default_expr {
            let default_func_ident = format_ident!("__default_{}_{}", ident, field_ident);

            default_functions = quote! {
                #default_functions

                #[allow(non_snake_case, dead_code)]
                fn #default_func_ident() -> #field_type {
                    #default_expr
                }
            };

            let default_func_ident_str = default_func_ident.to_string();

            quote! {
                #[serde(default = #default_func_ident_str)]
            }
        } else {
            quote!()
        };

        if attrs.is_asset {
            from_raw_body.extend(quote! {
                #field_ident: asset_server.load(raw.#field_ident),
            });

            fields.push(quote! {
                #default_attr
                #field_ident: String,
            });
        } else if attrs.is_id {
            from_raw_body.extend(quote! {
                #field_ident: raw.#field_ident.into(),
            });

            fields.push(quote! {
                #default_attr
                pub #field_ident: String,
            });
        } else {
            from_raw_body.extend(quote! {
                #field_ident: raw.#field_ident,
            });

            fields.push(quote! {
                #default_attr
                #field_ident: #field_type,
            });
        }
    }

    if id.is_none() {
        return input
            .span()
            .error("at least one field must be marked as id")
            .emit_as_expr_tokens()
            .into();
    }
    let id = id.unwrap();
    let generic_field = {
        let generics = &input.generics.params;

        quote! {
            _generic_marker: ::core::marker::PhantomData<(#generics)>
        }
    };

    let top_usage = if cfg!(feature = "schemars") {
        quote! { use ::schemars::*; }
    } else {
        quote!()
    };
    let raw_derives = if cfg!(feature = "schemars") {
        quote! { #[derive(Clone, ::serde::Serialize, ::serde::Deserialize, ::schemars::JsonSchema)] }
    } else {
        quote! { #[derive(Clone, ::serde::Serialize, ::serde::Deserialize)] }
    };

    quote! {
        const _: () = {
            #top_usage

            #default_functions

            impl #impl_generics ::bevy_histrion_proto::prototype::Prototype for #ident #ty_generics
            #where_clause
            {
                type Raw = #raw_prototype_ident;

                fn id(&self) -> Id<Self> {
                    #id
                }

                fn from_raw(raw: Self::Raw, asset_server: &::bevy::asset::AssetServer) -> Self {
                    Self {
                        #from_raw_body
                    }
                }

                fn discriminant() -> &'static str {
                    #discriminant
                }
            }

            #[allow(non_snake_case, dead_code)]
            #raw_derives
            pub struct #raw_prototype_ident #ty_generics
            #where_clause
            {
                #(#fields)*
                #[serde(skip)]
                #generic_field,
            }
        };
    }
    .into()
}
