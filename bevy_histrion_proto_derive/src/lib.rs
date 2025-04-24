mod attributes;

use std::collections::HashSet;

use attributes::SerdeAttributes;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Attribute, Data, DataEnum, DataStruct, DeriveInput, Expr, Fields, Lit, Meta, Token, Type,
    parse_macro_input, punctuated::Punctuated, spanned::Spanned,
};

#[proc_macro_derive(Prototype, attributes(proto))]
pub fn prototype_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let Data::Struct(struct_data) = &input.data else {
        return syn::Error::new(input.span(), "Prototype can only be derived for structs")
            .into_compile_error()
            .into();
    };

    if !matches!(struct_data.fields, Fields::Named(_)) {
        return syn::Error::new(
            input.span(),
            "Prototype can only be derived for structs with named fields",
        )
        .into_compile_error()
        .into();
    }

    let prototype_name = {
        let mut name = None;

        for attr in &input.attrs {
            if !attr.path().is_ident("proto") {
                continue;
            }

            let meta_list = match attr.meta.require_list() {
                Ok(list) => list,
                Err(err) => {
                    return err.into_compile_error().into();
                }
            };
            let meta_list =
                match meta_list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated) {
                    Ok(list) => list,
                    Err(err) => {
                        return err.into_compile_error().into();
                    }
                };

            for meta in &meta_list {
                if meta.path().is_ident("name") {
                    if name.is_some() {
                        return syn::Error::new(meta.span(), "Duplicate name attribute")
                            .into_compile_error()
                            .into();
                    }

                    let name_value = match meta.require_name_value() {
                        Ok(value) => value,
                        Err(err) => {
                            return err.into_compile_error().into();
                        }
                    };

                    let Expr::Lit(lit) = &name_value.value else {
                        return syn::Error::new(name_value.span(), "Name must be a string literal")
                            .into_compile_error()
                            .into();
                    };

                    let Lit::Str(lit_str) = &lit.lit else {
                        return syn::Error::new(name_value.span(), "Name must be a string literal")
                            .into_compile_error()
                            .into();
                    };

                    name = Some(lit_str.value());
                }
            }
        }

        if let Some(name) = name {
            name
        } else {
            return syn::Error::new(input.span(), "Prototype name is required")
                .into_compile_error()
                .into();
        }
    };

    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    quote! {
        impl #impl_generics ::bevy_histrion_proto::PrototypeData for #ident #ty_generics #where_clause {
            fn prototype_name() -> &'static str {
                #prototype_name
            }
        }
    }
    .into()
}

#[proc_macro_derive(JsonSchema, attributes(reflect, serde))]
pub fn json_schema_derive(item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as DeriveInput);

    let do_reflect_deserialize = do_reflect_deserialize(&item.attrs);
    let top_serde_attributes =
        match SerdeAttributes::try_from_attributes(&item.attrs, false, do_reflect_deserialize) {
            Ok(serde_attributes) => serde_attributes,
            Err(err) => {
                return err.into_compile_error().into();
            }
        };

    let body = match &item.data {
        Data::Struct(data_struct) => {
            json_schema_struct(data_struct, &top_serde_attributes, do_reflect_deserialize)
        }
        Data::Enum(data_enum) => {
            json_schema_enum(data_enum, &top_serde_attributes, do_reflect_deserialize)
        }
        _ => {
            return syn::Error::new(
                item.span(),
                "JsonSchema derive can only be applied to Struct or Enum",
            )
            .into_compile_error()
            .into();
        }
    };

    let body = match body {
        Ok(body) => body,
        Err(err) => {
            return err.into_compile_error().into();
        }
    };

    let ident = &item.ident;
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

    quote! {
        #[doc(hidden)]
        #[allow(
            non_upper_case_globals,
            unused_attributes,
            unused_qualifications,
            non_snake_case,
            dead_code,
            clippy::absolute_paths
        )]
        const _: () = {
            extern crate serde_json;

            impl #impl_generics JsonSchema for #ident #ty_generics #where_clause {
                fn json_schema(refs: &mut serde_json::Map<String, serde_json::Value>) -> serde_json::Value {
                    #body
                }
            }
        };
    }
    .into()
}

fn json_schema_struct(
    data_struct: &DataStruct,
    top_serde_attributes: &SerdeAttributes,
    do_reflect_deserialize: bool,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    match &data_struct.fields {
        Fields::Named(fields_named) => {
            let mut register_exp = quote!();
            let mut types = HashSet::new();
            let mut all_of = None;
            let mut properties = None;
            let mut required = quote!();

            for field in &fields_named.named {
                let serde_attributes = SerdeAttributes::try_from_attributes(
                    &field.attrs,
                    true,
                    do_reflect_deserialize,
                )?;

                if serde_attributes.skip {
                    continue;
                }

                let ty = &field.ty;

                if !types.contains(ty) {
                    types.insert(ty);
                    register_exp.extend(quote! {
                    let ty_title = <#ty as JsonSchema>::schema_title();
                    if !refs.contains_key(&ty_title) {
                        let ty_schema = <#ty as JsonSchema>::json_schema(refs);
                        refs.insert(ty_title, ty_schema);
                        }
                    });
                }

                if serde_attributes.flatten {
                    all_of.replace(quote! {
                        #all_of
                        { "$ref": <#ty as JsonSchema>::schema_ref() }
                    });
                    continue;
                }

                let ident = field.ident.clone().unwrap();
                let ident_str = if let Some(rename) = serde_attributes.rename {
                    rename
                } else if let Some(rename_all) = top_serde_attributes.rename_all_fields {
                    rename_all.apply_to_field(&ident.to_string())
                } else {
                    ident.to_string()
                };
                if !is_option(ty) && !serde_attributes.default {
                    required.extend(quote!(#ident_str,));
                }

                properties.replace(quote! {
                    #properties
                    #ident_str: { "$ref": <#ty as JsonSchema>::schema_ref() },
                });
            }

            let all_of = all_of.map_or(quote!(), |all_of| quote!("allOf": [#all_of],));
            let properties =
                properties.map_or(quote!(), |properties| quote!("properties": {#properties},));
            Ok(quote! {
                #register_exp
                let schema = serde_json::json!({
                    "type": "object",
                    "required": [#required],
                    #properties
                    #all_of
                });

                schema
            })
        }
        Fields::Unnamed(fields_unnamed) => {
            let mut register_exp = quote!();
            let mut refs = quote!();
            let mut types = HashSet::new();
            let mut num_fields = 0;

            for field in &fields_unnamed.unnamed {
                let serde_attributes = SerdeAttributes::try_from_attributes(
                    &field.attrs,
                    true,
                    do_reflect_deserialize,
                )?;

                if serde_attributes.skip {
                    continue;
                }

                num_fields += 1;
                let ty = &field.ty;

                refs.extend(quote! {
                    { "$refs": <#ty as JsonSchema>::schema_ref() },
                });

                if !types.contains(ty) {
                    types.insert(ty);
                    register_exp.extend(quote! {
                        let ty_title = <#ty as JsonSchema>::schema_title();
                        if !refs.contains_key(&ty_title) {
                            let ty_schema = <#ty as JsonSchema>::json_schema(refs);
                            refs.insert(ty_title, ty_schema);
                        }
                    });
                }
            }

            Ok(quote! {
                #register_exp

                serde_json::json!({
                    "type": "array",
                    "items": [
                        #refs
                    ],
                    "minItems": #num_fields,
                    "maxItems": #num_fields,
                })
            })
        }
        Fields::Unit => Ok(quote!(serde_json::json!({
            "type": "null"
        }))),
    }
}

fn json_schema_enum(
    data_enum: &DataEnum,
    top_serde_attributes: &SerdeAttributes,
    do_reflect_deserialize: bool,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    let mut register_exp = quote!();
    let mut one_of = quote!();
    let mut types = HashSet::new();

    for variant in &data_enum.variants {
        let serde_attributes =
            SerdeAttributes::try_from_attributes(&variant.attrs, true, do_reflect_deserialize)?;

        let ident = variant.ident.clone();
        let variant_name_str = if let Some(rename) = serde_attributes.rename {
            rename
        } else if let Some(rename_all) = top_serde_attributes.rename_all {
            rename_all.apply_to_variant(&ident.to_string())
        } else {
            ident.to_string()
        };
        match &variant.fields {
            Fields::Named(fields_named) => {
                let mut all_of = None;
                let mut properties = None;
                let mut required = quote!();

                for field in &fields_named.named {
                    let serde_attributes = SerdeAttributes::try_from_attributes(
                        &field.attrs,
                        true,
                        do_reflect_deserialize,
                    )?;

                    if serde_attributes.skip {
                        continue;
                    }

                    let ty = &field.ty;

                    if !types.contains(ty) {
                        types.insert(ty);
                        register_exp.extend(quote! {
                        let ty_title = <#ty as JsonSchema>::schema_title();
                        if !refs.contains_key(&ty_title) {
                            let ty_schema = <#ty as JsonSchema>::json_schema(refs);
                            refs.insert(ty_title, ty_schema);
                            }
                        });
                    }

                    if serde_attributes.flatten {
                        all_of.replace(quote! {
                            #all_of
                            { "$ref": <#ty as JsonSchema>::schema_ref() }
                        });
                        continue;
                    }

                    let field_ident = field.ident.clone().unwrap();
                    let field_name = if let Some(rename) = serde_attributes.rename {
                        rename
                    } else if let Some(rename_all) = top_serde_attributes.rename_all_fields {
                        rename_all.apply_to_field(&field_ident.to_string())
                    } else {
                        field_ident.to_string()
                    };
                    if !is_option(ty) && !serde_attributes.default {
                        required.extend(quote!(#field_name,));
                    }

                    properties.replace(quote! {
                        #properties
                        #field_name: { "$ref": <#ty as JsonSchema>::schema_ref() },
                    });
                }

                let all_of = all_of.map_or(quote!(), |all_of| quote!("allOf": [#all_of],));
                let properties =
                    properties.map_or(quote!(), |properties| quote!("properties": {#properties},));
                one_of.extend(quote! {
                    {
                        "type": "object",
                        "required": [#required],
                        "properties": {
                            #variant_name_str: {
                                "type": "object",
                                #all_of
                                #properties
                            }
                        }
                    },
                });
            }
            Fields::Unnamed(fields_unnamed) => {
                let mut refs = quote!();
                let mut num_fields = 0;

                for field in &fields_unnamed.unnamed {
                    let serde_attributes = SerdeAttributes::try_from_attributes(
                        &field.attrs,
                        true,
                        do_reflect_deserialize,
                    )?;

                    if serde_attributes.skip {
                        continue;
                    }

                    num_fields += 1;
                    let ty = &field.ty;

                    if !types.contains(ty) {
                        types.insert(ty);
                        register_exp.extend(quote! {
                        let ty_title = <#ty as JsonSchema>::schema_title();
                            if !refs.contains_key(&ty_title) {
                                let ty_schema = <#ty as JsonSchema>::json_schema(refs);
                                refs.insert(ty_title, ty_schema);
                            }
                        });
                    }

                    refs.extend(quote! {
                        { "$refs": <#ty as JsonSchema>::schema_ref() },
                    });
                }

                one_of.extend(quote! {
                    {
                        "type": "object",
                        "properties": {
                            #variant_name_str: {
                                "type": "array",
                                "items": {
                                    #refs
                                },
                                "minItems": #num_fields,
                                "maxItems": #num_fields,
                            }
                        }
                    },
                });
            }
            Fields::Unit => {
                one_of.extend(quote! {
                    { "type": "string", "enum": [#variant_name_str] },
                });
            }
        }
    }

    Ok(quote! {
        #register_exp
        serde_json::json!({
            "type": "object",
            "oneOf": [#one_of],
        })
    })
}

fn is_option(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Option";
        }
    }
    false
}

fn do_reflect_deserialize(attrs: &[Attribute]) -> bool {
    for attr in attrs {
        if !attr.path().is_ident("reflect") {
            continue;
        }

        let Some(meta_list) = attr.meta.require_list().ok() else {
            continue;
        };
        let Some(meta_list) = meta_list
            .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
            .ok()
        else {
            continue;
        };

        for meta in meta_list {
            if meta.path().is_ident("Deserialize") {
                return true;
            }
        }
    }

    false
}
