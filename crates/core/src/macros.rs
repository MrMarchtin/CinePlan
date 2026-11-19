/// Déclare une énumération stockée sous forme de texte (SQLite, JSON).
/// Génère `as_str()`, `FromStr` et les attributs serde correspondants.
macro_rules! str_enum {
    ($(#[$meta:meta])* $name:ident { $($variant:ident => $s:literal),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        pub enum $name {
            $(#[serde(rename = $s)] $variant),+
        }

        impl $name {
            pub fn as_str(&self) -> &'static str {
                match self { $(Self::$variant => $s),+ }
            }
        }

        impl std::str::FromStr for $name {
            type Err = $crate::CoreError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                match value {
                    $($s => Ok(Self::$variant),)+
                    other => Err($crate::CoreError::Storage(format!(
                        "valeur inconnue pour {} : {}",
                        stringify!($name),
                        other
                    ))),
                }
            }
        }
    };
}
