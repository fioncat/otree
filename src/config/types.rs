use serde::{Deserialize, Serialize};

macro_rules! generate_types_default {
    ($StructName:ident, $($field:ident),+) => {
        impl $StructName {
            ::paste::paste! {
                pub fn default() -> Self {
                    Self {
                        $(
                            $field: Self::[<default_ $field>](),
                        )+
                    }
                }
            }

            $(
                ::paste::paste! {
                    pub fn [<default_ $field>]() -> String {
                        String::from(stringify!($field))
                    }
                }
            )+
        }
    };
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Types {
    #[serde(default = "Types::default_str")]
    pub str: String,

    #[serde(default = "Types::default_null")]
    pub null: String,

    #[serde(default = "Types::default_bool")]
    pub bool: String,

    #[serde(default = "Types::default_num")]
    pub num: String,

    #[serde(default = "Types::default_arr")]
    pub arr: String,

    #[serde(default = "Types::default_obj")]
    pub obj: String,
}

generate_types_default!(Types, str, null, bool, num, arr, obj);
