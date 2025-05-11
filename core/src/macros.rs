/// Creates a macro for converting different integer types into [Category][category].
///
/// [category]: crate::category::Category
macro_rules! category_try_from_integers {
    ($($typ: ident),*) => {
        $(
            impl ::std::convert::TryFrom<$typ> for crate::detector::category::Category {
                type Error = crate::error::Error;

                fn try_from(
                    value: ::core::primitive::$typ
                ) -> ::std::result::Result<
                    crate::detector::category::Category,
                    crate::error::Error
                > {
                    match value {
                        1 => Ok(crate::detector::category::Category::Animal),
                        2 => Ok(crate::detector::category::Category::Human),
                        3 => Ok(crate::detector::category::Category::Vehicle),
                        _other => Err(crate::error::Error::CategoryIndexOutOfRange(_other as f64)),
                    }
                }
            }
        )*
    };
}

/// Creates a macro for converting different float types into [Category][category].
///
/// [category]: crate::category::Category
macro_rules! category_try_from_floats {
    ($($typ: ident),*) => {
        $(
            impl ::std::convert::TryFrom<$typ> for crate::detector::category::Category {
                type Error = crate::error::Error;

                fn try_from(
                    value: ::core::primitive::$typ
                ) -> ::std::result::Result<
                    crate::detector::category::Category,
                    crate::error::Error
                > {
                    match value {
                        1.0 => Ok(crate::detector::category::Category::Animal),
                        2.0 => Ok(crate::detector::category::Category::Human),
                        3.0 => Ok(crate::detector::category::Category::Vehicle),
                        _other => Err(crate::error::Error::CategoryIndexOutOfRange(_other as f64)),
                    }
                }
            }
        )*
    };
}

pub(crate) use category_try_from_floats;
pub(crate) use category_try_from_integers;
