/// Declares an enum stored and serialised by name, with Java's `name()`, `ordinal()` and `valueOf()`.
macro_rules! string_enum {
    ($(#[$meta:meta])* $name:ident { $($variant:ident = $text:literal),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
        #[sea_orm(rs_type = "String", db_type = "String(StringLen::N(20))")]
        pub enum $name {
            $(
                #[sea_orm(string_value = $text)]
                #[serde(rename = $text)]
                $variant,
            )+
        }

        impl $name {
            /// `Enum.name()`.
            pub fn name(&self) -> &'static str {
                match self { $(Self::$variant => $text,)+ }
            }

            /// `Enum.ordinal()`.
            pub fn ordinal(&self) -> usize {
                <Self as sea_orm::Iterable>::iter().position(|v| v == *self).unwrap_or(0)
            }

            /// `Enum.valueOf(name)`.
            pub fn value_of(name: &str) -> Option<Self> {
                match name { $($text => Some(Self::$variant),)+ _ => None }
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.name())
            }
        }
    };
}
