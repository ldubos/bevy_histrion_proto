use proc_macro2_diagnostics::Diagnostic;
use syn::{Attribute, Expr, LitStr, Token, parenthesized};

#[derive(Default)]
pub struct PrototypeAttributes {
    pub is_id: bool,
    pub is_asset: bool,
    pub discriminant: Option<String>,
    pub default_func: Option<String>,
    pub default_expr: Option<Expr>,
}

pub fn parse_attributes(
    attr: &Attribute,
    is_field: bool,
    prototype_attributes: &mut PrototypeAttributes,
) -> Result<(), Diagnostic> {
    attr.parse_nested_meta(|nested_meta| match nested_meta.path.get_ident() {
        Some(ident) => match ident.to_string().as_str() {
            "id" => {
                if !is_field {
                    return Err(nested_meta.error("id attribute can only be used on fields"));
                }

                if prototype_attributes.is_asset
                    || (prototype_attributes.default_func.is_some()
                        || prototype_attributes.default_expr.is_some())
                {
                    return Err(nested_meta
                        .error("id attribute cannot be used with asset or default attributes"));
                }

                if prototype_attributes.is_id {
                    return Err(nested_meta
                        .error("id attribute cannot be used more than once on the same field"));
                }

                if !nested_meta.input.is_empty() {
                    return Err(nested_meta.error("id attribute cannot have any arguments"));
                }

                prototype_attributes.is_id = true;

                Ok(())
            }
            "asset" => {
                if !is_field {
                    return Err(nested_meta.error("asset attribute can only be used on fields"));
                }

                if prototype_attributes.is_id {
                    return Err(
                        nested_meta.error("asset attribute cannot be used with id attribute")
                    );
                }

                if prototype_attributes.is_asset {
                    return Err(nested_meta
                        .error("asset attribute cannot be used more than once on the same field"));
                }

                if !nested_meta.input.is_empty() {
                    return Err(nested_meta.error("asset attribute cannot have any arguments"));
                }

                prototype_attributes.is_asset = true;

                Ok(())
            }
            "discriminant" => {
                if is_field {
                    return Err(
                        nested_meta.error("discriminant attribute cannot be used on fields")
                    );
                }

                if prototype_attributes.discriminant.is_some() {
                    return Err(
                        nested_meta.error("discriminant attribute cannot be used more than once")
                    );
                }

                nested_meta.input.parse::<Token![=]>()?;

                let discriminant = nested_meta.input.parse::<LitStr>()?.value();
                prototype_attributes.discriminant = Some(discriminant);

                Ok(())
            }
            "default" => {
                if !is_field {
                    return Err(nested_meta.error("default attribute can only be used on fields"));
                }

                if prototype_attributes.is_id {
                    return Err(
                        nested_meta.error("default attribute cannot be used with id attribute")
                    );
                }

                if prototype_attributes.default_expr.is_some()
                    || prototype_attributes.default_func.is_some()
                {
                    return Err(nested_meta.error(
                        "default attribute cannot be used more than once on the same field",
                    ));
                }

                if nested_meta.input.parse::<Token![=]>().is_ok() {
                    let default = nested_meta.input.parse::<LitStr>()?.value();
                    prototype_attributes.default_func = Some(default);
                } else {
                    let content;
                    parenthesized!(content in nested_meta.input);
                    let default = content.parse::<Expr>()?;
                    prototype_attributes.default_expr = Some(default);
                }

                Ok(())
            }
            attr => Err(nested_meta.error(format!("unknown prototype attribute: {}", attr))),
        },
        _ => Ok(()), /* ignore no sub-attributes */
    })?;

    Ok(())
}
