use bevy::reflect::TypePath;
use serde_json::{Map as JsonMap, Value as JsonValue, json};

use crate::PrototypeData;

pub trait JsonSchema: TypePath {
    fn json_schema(refs: &mut JsonMap<String, JsonValue>) -> JsonValue;

    fn schema_title() -> String {
        Self::type_path().to_string()
    }

    fn schema_ref() -> String {
        format!("#/definitions/{}", Self::schema_title())
    }
}

macro_rules! impl_schema_for_int {
    ($t:ty, $comment:literal) => {
        impl JsonSchema for $t {
            fn json_schema(_refs: &mut JsonMap<String, JsonValue>) -> JsonValue {
                json!({
                    "type": "integer",
                    "default": <$t as Default>::default(),
                    "$comment": $comment,
                    "minimum": <$t>::MIN,
                    "maximum": <$t>::MAX,
                    "format": stringify!($t)
                })
            }
        }
    };
    ($({$t:ty, $comment:literal}),+) => {
        $(
            impl_schema_for_int!($t, $comment);
        )+
    }
}

impl_schema_for_int!(
    {u8, "8-bit unsigned integer"},
    {u16, "16-bit unsigned integer"},
    {u32, "32-bit unsigned integer"},
    {u64, "64-bit unsigned integer"},
    {u128, "128-bit unsigned integer"},
    {i8, "8-bit signed integer"},
    {i16, "16-bit signed integer"},
    {i32, "32-bit signed integer"},
    {i64, "64-bit signed integer"},
    {i128, "128-bit signed integer"}
);

macro_rules! impl_schema_for_non_zero_int {
    ($t:ty, $comment:literal, $format:ty) => {
        impl JsonSchema for $t {
            fn json_schema(_refs: &mut JsonMap<String, JsonValue>) -> JsonValue {
                json!({
                    "type": "integer",
                    "$comment": $comment,
                    "default": 1,
                    "minimum": 1,
                    "maximum": <$t>::MAX,
                    "format": stringify!($format)
                })
            }
        }
    };
    ($({$t:ty, $comment:literal, $format:ty}),+) => {
        $(
            impl_schema_for_non_zero_int!($t, $comment, $format);
        )+
    }
}

impl_schema_for_non_zero_int!(
    {::core::num::NonZeroU8, "non-zero 8-bit unsigned integer", u8},
    {::core::num::NonZeroU16, "non-zero 16-bit unsigned integer", u16},
    {::core::num::NonZeroU32, "non-zero 32-bit unsigned integer", u32},
    {::core::num::NonZeroU64, "non-zero 64-bit unsigned integer", u64},
    {::core::num::NonZeroU128, "non-zero 128-bit unsigned integer", u128},
    {::core::num::NonZeroI8, "non-zero 8-bit signed integer", i8},
    {::core::num::NonZeroI16, "non-zero 16-bit signed integer", i16},
    {::core::num::NonZeroI32, "non-zero 32-bit signed integer", i32},
    {::core::num::NonZeroI64, "non-zero 64-bit signed integer", i64},
    {::core::num::NonZeroI128, "non-zero 128-bit signed integer", i128}
);

impl JsonSchema for f32 {
    fn json_schema(_refs: &mut JsonMap<String, JsonValue>) -> JsonValue {
        json!({
            "type": "number",
            "default": 0.0,
            "$comment": "32-bit floating point number",
            "minimum": f32::MIN,
            "maximum": f32::MAX,
            "format": "double"
        })
    }
}

impl JsonSchema for f64 {
    fn json_schema(_refs: &mut JsonMap<String, JsonValue>) -> JsonValue {
        json!({
            "type": "number",
            "default": 0.0,
            "$comment": "64-bit floating point number",
            "minimum": f64::MIN,
            "maximum": f64::MAX,
            "format": "double"
        })
    }
}

impl JsonSchema for () {
    fn json_schema(_refs: &mut JsonMap<String, JsonValue>) -> JsonValue {
        json!({
            "type": "null",
        })
    }
}

impl JsonSchema for bool {
    fn json_schema(_refs: &mut JsonMap<String, JsonValue>) -> JsonValue {
        json!({
            "type": "boolean",
        })
    }
}

impl JsonSchema for char {
    fn json_schema(_refs: &mut JsonMap<String, JsonValue>) -> JsonValue {
        json!({
            "type": "string",
            "$comment": "single character",
            "minLength": 1,
            "maxLength": 1,
        })
    }
}

impl JsonSchema for String {
    fn json_schema(_refs: &mut JsonMap<String, JsonValue>) -> JsonValue {
        json!({
            "type": "string",
        })
    }
}

impl<A: ::bevy::asset::Asset> JsonSchema for ::bevy::asset::Handle<A> {
    fn json_schema(_refs: &mut JsonMap<String, JsonValue>) -> JsonValue {
        json!({
            "type": "string",
            "$comment": "an asset path",
        })
    }
}

impl JsonSchema for ::bevy::asset::AssetPath<'static> {
    fn json_schema(_refs: &mut JsonMap<String, JsonValue>) -> JsonValue {
        json!({
            "type": "string",
            "$comment": "an asset path",
        })
    }
}

impl JsonSchema for std::path::PathBuf {
    fn json_schema(_refs: &mut JsonMap<String, JsonValue>) -> JsonValue {
        json!({
            "type": "string",
            "$comment": "path",
        })
    }
}

macro_rules! impl_schema_for_vec {
    ($ty:ty, $scalar:ty, $arity:literal, $name:literal, $comment:literal) => {
        impl JsonSchema for $ty {
            fn json_schema(refs: &mut JsonMap<String, JsonValue>) -> JsonValue {
                let scalar_title = <$scalar as JsonSchema>::schema_title();

                if !refs.contains_key(&scalar_title) {
                    let scalar_schema = <$scalar as JsonSchema>::json_schema(refs);
                    refs.insert(scalar_title, scalar_schema);
                }

                json!({
                    "type": "array",
                    "items": {
                        "$ref": <$scalar as JsonSchema>::schema_ref()
                    },
                    "$comment": $comment,
                    "default": vec![<$scalar as Default>::default(); $arity],
                    "minItems": $arity,
                    "maxItems": $arity,
                })
            }

            fn schema_title() -> String {
                $name.to_string()
            }
        }
    };
    ($({$ty:ty, $scalar:ty, $arity:literal, $name:literal, $comment:literal}),+) => {
        $(
            impl_schema_for_vec!($ty, $scalar, $arity, $name, $comment);
        )+
    }
}

impl_schema_for_vec!(
    {::bevy::math::U8Vec2, u8, 2, "U8Vec2", "2D vector of u8"},
    {::bevy::math::U8Vec3, u8, 3, "U8Vec3", "3D vector of u8"},
    {::bevy::math::U8Vec4, u8, 4, "U8Vec4", "4D vector of u8"},
    {::bevy::math::U16Vec2, u16, 2, "U16Vec2", "2D vector of u16"},
    {::bevy::math::U16Vec3, u16, 3, "U16Vec3", "3D vector of u16"},
    {::bevy::math::U16Vec4, u16, 4, "U16Vec4", "4D vector of u16"},
    {::bevy::math::UVec2, u32, 2, "UVec2", "2D vector of u32"},
    {::bevy::math::UVec3, u32, 3, "UVec3", "3D vector of u32"},
    {::bevy::math::UVec4, u32, 4, "UVec4", "4D vector of u32"},
    {::bevy::math::U64Vec2, u64, 2, "U64Vec2", "2D vector of u64"},
    {::bevy::math::U64Vec3, u64, 3, "U64Vec3", "3D vector of u64"},
    {::bevy::math::U64Vec4, u64, 4, "U64Vec4", "4D vector of u64"},
    {::bevy::math::I8Vec2, i8, 2, "I8Vec2", "2D vector of i8"},
    {::bevy::math::I8Vec3, i8, 3, "I8Vec3", "3D vector of i8"},
    {::bevy::math::I8Vec4, i8, 4, "I8Vec4", "4D vector of i8"},
    {::bevy::math::I16Vec2, i16, 2, "I16Vec2", "2D vector of i16"},
    {::bevy::math::I16Vec3, i16, 3, "I16Vec3", "3D vector of i16"},
    {::bevy::math::I16Vec4, i16, 4, "I16Vec4", "4D vector of i16"},
    {::bevy::math::IVec2, i32, 2, "IVec2", "2D vector of i32"},
    {::bevy::math::IVec3, i32, 3, "IVec3", "3D vector of i32"},
    {::bevy::math::IVec4, i32, 4, "IVec4", "4D vector of i32"},
    {::bevy::math::I64Vec2, i64, 2, "I64Vec2", "2D vector of i64"},
    {::bevy::math::I64Vec3, i64, 3, "I64Vec3", "3D vector of i64"},
    {::bevy::math::I64Vec4, i64, 4, "I64Vec4", "4D vector of i64"},
    {::bevy::math::Vec2, f32, 2, "Vec2", "2D vector of f32"},
    {::bevy::math::Vec3, f32, 3, "Vec3", "3D vector of f32"},
    {::bevy::math::Vec3A, f32, 3, "Vec3", "3D vector of f32"},
    {::bevy::math::Vec4, f32, 4, "Vec4", "4D vector of f32"},
    {::bevy::math::BVec2, bool, 2, "BVec2", "2D vector of bool"},
    {::bevy::math::BVec3, bool, 3, "BVec3", "3D vector of bool"},
    {::bevy::math::BVec4, bool, 4, "BVec4", "4D vector of bool"},
    {::bevy::math::Mat2, ::bevy::math::Vec2, 2, "Mat2", "2x2 matrix of f32"},
    {::bevy::math::Mat3, ::bevy::math::Vec3, 3, "Mat3", "3x3 matrix of f32"},
    {::bevy::math::Mat3A, ::bevy::math::Vec3, 3, "Mat3", "3x3 matrix of f32"},
    {::bevy::math::Mat4, ::bevy::math::Vec4, 4, "Mat4", "4x4 matrix of f32"},
    {::bevy::math::Dir2, f32, 2, "Direction2d", "2D direction vector of f32"},
    {::bevy::math::Dir3, f32, 3, "Direction3d", "3D direction vector of f32"},
    {::bevy::math::Dir3A, f32, 3, "Direction3d", "3D direction vector of f32"}
);

impl<T: JsonSchema> JsonSchema for Option<T>
where
    Option<T>: TypePath,
{
    fn json_schema(refs: &mut JsonMap<String, JsonValue>) -> JsonValue {
        let t_title = <T as JsonSchema>::schema_title();

        if !refs.contains_key(&t_title) {
            let t_schema = <T as JsonSchema>::json_schema(refs);
            refs.insert(t_title, t_schema);
        }

        json!({
            "type": [
                "object",
                "null"
            ],
            "$ref": <T as JsonSchema>::schema_ref(),
            "$comment": "optional value"
        })
    }

    fn schema_title() -> String {
        format!("Option<{}>", <T as JsonSchema>::schema_title())
    }
}

impl<T: JsonSchema> JsonSchema for Vec<T>
where
    Vec<T>: TypePath,
{
    fn json_schema(refs: &mut JsonMap<String, JsonValue>) -> JsonValue {
        let t_title = <T as JsonSchema>::schema_title();

        if !refs.contains_key(&t_title) {
            let t_schema = <T as JsonSchema>::json_schema(refs);
            refs.insert(t_title, t_schema);
        }

        json!({
            "type": "array",
            "items": { "$ref": <T as JsonSchema>::schema_ref() },
        })
    }

    fn schema_title() -> String {
        format!("Vec<{}>", <T as JsonSchema>::schema_title())
    }
}

impl<T: JsonSchema, const N: usize> JsonSchema for [T; N]
where
    [T; N]: TypePath,
{
    fn json_schema(refs: &mut JsonMap<String, JsonValue>) -> JsonValue {
        let t_title = <T as JsonSchema>::schema_title();

        if !refs.contains_key(&t_title) {
            let t_schema = <T as JsonSchema>::json_schema(refs);
            refs.insert(t_title, t_schema);
        }

        json!({
            "type": "array",
            "items": { "$ref": <T as JsonSchema>::schema_ref() },
            "minItems": N,
            "maxItems": N,
        })
    }
}

macro_rules! impl_schema_for_tuple {
    ($N:expr, $($T:ident),*) => {
        impl<$($T: JsonSchema),*> JsonSchema for ($($T,)*) where ($($T,)*): TypePath {
            fn json_schema(refs: &mut JsonMap<String, JsonValue>) -> JsonValue {
                $(
                    let t_title = <$T as JsonSchema>::schema_title();

                    if !refs.contains_key(&t_title) {
                        let t_schema = <$T as JsonSchema>::json_schema(refs);
                        refs.insert(t_title, t_schema);
                    }
                )*

                json!({
                    "type": "array",
                    "items": [
                        $({ "$ref": <$T as JsonSchema>::schema_ref() }),*
                    ],
                    "minItems": $N,
                    "maxItems": $N,
                })
            }

            fn schema_title() -> String {
                format!("({})", [$(<$T as JsonSchema>::schema_title(),)*].join(", "))
            }
        }
    }
}

variadics_please::all_tuples_with_size!(impl_schema_for_tuple, 1, 15, T);

impl JsonSchema for core::time::Duration {
    fn json_schema(_refs: &mut JsonMap<String, JsonValue>) -> JsonValue {
        json!({
            "type": "string",
            "format": "duration",
        })
    }
}

impl<P: PrototypeData> JsonSchema for crate::identifier::PrototypeId<P> {
    fn json_schema(_refs: &mut JsonMap<String, JsonValue>) -> JsonValue {
        json!({
            "type": "string",
            "default": "",
            "$comment": "an identifier for a prototype",
        })
    }

    fn schema_title() -> String {
        String::from("PrototypeId")
    }
}

impl<P: PrototypeData> JsonSchema for crate::identifier::PrototypeName<P> {
    fn json_schema(_refs: &mut JsonMap<String, JsonValue>) -> JsonValue {
        json!({
            "type": "string",
            "default": "",
            "$comment": "an identifier for a prototype",
        })
    }

    fn schema_title() -> String {
        String::from("PrototypeName")
    }
}
