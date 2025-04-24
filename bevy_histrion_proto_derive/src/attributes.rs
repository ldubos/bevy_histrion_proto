use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{Attribute, Expr, Lit, Meta, Token, punctuated::Punctuated};

#[derive(Default, Clone)]
pub(crate) struct SerdeAttributes {
    pub flatten: bool,
    pub skip: bool,
    pub rename: Option<String>,
    pub untagged: bool,
    pub tag: Option<TokenStream>,
    pub content: Option<TokenStream>,
    pub rename_all: Option<SerdeRenameAll>,
    pub rename_all_fields: Option<SerdeRenameAll>,
    pub default: bool,
}

impl SerdeAttributes {
    pub fn try_from_attributes(
        attrs: &[Attribute],
        is_field: bool,
        do_reflect_deserialize: bool,
    ) -> Result<Self, syn::Error> {
        let mut serde_attributes = SerdeAttributes::default();

        if !do_reflect_deserialize {
            return Ok(serde_attributes);
        }

        for attr in attrs {
            if !attr.path().is_ident("serde") {
                continue;
            }

            let meta_list = attr
                .meta
                .require_list()
                .map_err(|err| syn::Error::new(err.span(), format!("{err}")))?;
            let meta_list = meta_list
                .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                .map_err(|err| syn::Error::new(err.span(), format!("{err}")))?;

            if !is_field {
                for meta in &meta_list {
                    if meta.path().is_ident("untagged") {
                        serde_attributes.untagged = true;
                    } else if meta.path().is_ident("tag") {
                        let Some(name_value) = meta.require_name_value().ok() else {
                            continue;
                        };

                        let Expr::Lit(lit) = &name_value.value else {
                            continue;
                        };

                        let Lit::Str(lit_str) = &lit.lit else {
                            continue;
                        };

                        serde_attributes.tag.replace(lit_str.to_token_stream());
                    } else if meta.path().is_ident("content") {
                        let Some(name_value) = meta.require_name_value().ok() else {
                            continue;
                        };

                        let Expr::Lit(lit) = &name_value.value else {
                            continue;
                        };

                        let Lit::Str(lit_str) = &lit.lit else {
                            continue;
                        };

                        serde_attributes.content.replace(lit_str.to_token_stream());
                    } else if meta.path().is_ident("rename_all") {
                        serde_attributes.rename_all = SerdeRenameAll::try_from_meta(meta);
                    } else if meta.path().is_ident("rename_all_fields") {
                        serde_attributes.rename_all_fields = SerdeRenameAll::try_from_meta(meta);
                    }
                }
            } else {
                for meta in &meta_list {
                    if meta.path().is_ident("skip") {
                        serde_attributes.skip = true;
                    } else if meta.path().is_ident("flatten") {
                        serde_attributes.flatten = true;
                    } else if meta.path().is_ident("rename") {
                        let Some(name_value) = meta.require_name_value().ok() else {
                            continue;
                        };

                        let Expr::Lit(lit) = &name_value.value else {
                            continue;
                        };

                        let Lit::Str(lit_str) = &lit.lit else {
                            continue;
                        };

                        serde_attributes.rename.replace(lit_str.value());
                    } else if meta.path().is_ident("default") {
                        serde_attributes.default = true;
                    }
                }
            }
        }

        Ok(serde_attributes)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum SerdeRenameAll {
    LowerCase,
    UpperCase,
    PascalCase,
    CamelCase,
    SnakeCase,
    ScreamingSnakeCase,
    KebabCase,
    ScreamingKebabCase,
}

impl SerdeRenameAll {
    pub fn try_from_meta(meta: &Meta) -> Option<SerdeRenameAll> {
        let name_value = meta.require_name_value().ok()?;
        let Expr::Lit(lit) = &name_value.value else {
            return None;
        };
        let Lit::Str(lit_str) = &lit.lit else {
            return None;
        };

        match lit_str.value().as_str() {
            "lowercase" => Some(SerdeRenameAll::LowerCase),
            "UPPERCASE" => Some(SerdeRenameAll::UpperCase),
            "PascalCase" => Some(SerdeRenameAll::PascalCase),
            "camelCase" => Some(SerdeRenameAll::CamelCase),
            "snake_case" => Some(SerdeRenameAll::SnakeCase),
            "SCREAMING_SNAKE_CASE" => Some(SerdeRenameAll::ScreamingSnakeCase),
            "kebab-case" => Some(SerdeRenameAll::KebabCase),
            "SCREAMING-KEBAB-CASE" => Some(SerdeRenameAll::ScreamingKebabCase),
            _ => None,
        }
    }

    pub fn apply_to_variant(self, variant: &str) -> String {
        use SerdeRenameAll::*;

        match self {
            PascalCase => variant.to_owned(),
            LowerCase => variant.to_ascii_lowercase(),
            UpperCase => variant.to_ascii_uppercase(),
            CamelCase => variant[..1].to_ascii_lowercase() + &variant[1..],
            SnakeCase => {
                let mut snake = String::new();
                for (i, ch) in variant.char_indices() {
                    if i > 0 && ch.is_uppercase() {
                        snake.push('_');
                    }
                    snake.push(ch.to_ascii_lowercase());
                }
                snake
            }
            ScreamingSnakeCase => SnakeCase.apply_to_variant(variant).to_ascii_uppercase(),
            KebabCase => SnakeCase.apply_to_variant(variant).replace('_', "-"),
            ScreamingKebabCase => ScreamingSnakeCase
                .apply_to_variant(variant)
                .replace('_', "-"),
        }
    }

    pub fn apply_to_field(self, field: &str) -> String {
        use SerdeRenameAll::*;

        match self {
            LowerCase | SnakeCase => field.to_owned(),
            UpperCase | ScreamingSnakeCase => field.to_ascii_uppercase(),
            PascalCase => {
                let mut pascal = String::new();
                let mut capitalize = true;
                for ch in field.chars() {
                    if ch == '_' {
                        capitalize = true;
                    } else if capitalize {
                        pascal.push(ch.to_ascii_uppercase());
                        capitalize = false;
                    } else {
                        pascal.push(ch);
                    }
                }
                pascal
            }
            CamelCase => {
                let pascal = PascalCase.apply_to_field(field);
                pascal[..1].to_ascii_lowercase() + &pascal[1..]
            }
            KebabCase => field.replace('_', "-"),
            ScreamingKebabCase => ScreamingSnakeCase.apply_to_field(field).replace('_', "-"),
        }
    }
}
