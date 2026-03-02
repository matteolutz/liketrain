#[macro_export]
macro_rules! deser_variant {
    (
        $name:ident {
            $(
                $variant:ident = $variant_value:expr
            ),* $(,)?
        }
    ) => {
        #[derive(Debug, Copy, Clone)]
        #[repr(u8)]
        pub enum $name {
            $(
                $variant = $variant_value,
            )*
        }

        impl From<$name> for u8 {
            fn from(value: $name) -> u8 {
                value as u8
            }
        }

        impl TryFrom<u8> for $name {
            type Error = ();

            fn try_from(value: u8) -> Result<Self, Self::Error> {
                match value {
                    $(
                        $variant_value => Ok($name::$variant),
                    )*
                    _ => Err(()),
                }
            }
        }
    };
}
