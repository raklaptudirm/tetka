#[macro_export]
#[doc(hidden)]
macro_rules! impl_from_integer_for_enum {
    (for $type:ident: $($num:ident, $fn:path;)*) => {$(
        impl TryFrom<$num> for $type {
            type Error = ();
            #[inline(always)]
            fn try_from(number: $num) -> Result<Self, Self::Error> {
                match $fn(number) {
                    Some(n) => Ok(n),
                    None => Err(()),
                }
            }
        }

        impl From<$type> for $num {
            #[inline(always)]
            fn from(number: $type) -> Self {
                number as Self
            }
        }
    )*};
}

pub use impl_from_integer_for_enum;

#[macro_export]
#[doc(hidden)]
macro_rules! impl_unary_ops_for_tuple {
    (for $type:ident: $($trait:path, $fn:ident, $op:tt;)*) => {$(
        impl $trait for $type {
            type Output = Self;

            #[inline(always)]
            fn $fn(self) -> Self::Output {
                Self($op self.0)
            }
        }
    )*};
}

pub use impl_unary_ops_for_tuple;

#[macro_export]
#[doc(hidden)]
macro_rules! impl_binary_ops_for_tuple {
    (for $type:ident: $($trait:path, $fn:ident, $op:tt;)*) => {$(
        impl $trait for $type {
            type Output = Self;

            #[inline(always)]
            fn $fn(self, rhs: Self) -> Self::Output {
                Self(self.0 $op rhs.0)
            }
        }
    )*};
}

pub use impl_binary_ops_for_tuple;

#[macro_export]
#[doc(hidden)]
macro_rules! impl_assign_ops_for_tuple {
    (for $type:ident: $($trait:path, $fn:ident, $op:tt;)*) => {$(
        impl $trait for $type {
            #[inline(always)]
            fn $fn(&mut self, rhs: Self) {
                self.0 = self.0 $op rhs.0
            }
        }
    )*};
}

pub use impl_assign_ops_for_tuple;

#[macro_export]
#[doc(hidden)]
macro_rules! impl_from_integer_for_tuple {
    (for $type:ident $root_type:ident: $($num:ident,)*) => {$(
        impl From<$num> for $type {
            #[inline(always)]
            fn from(number: $num) -> Self {
                Self(number as $root_type)
            }
        }

        impl From<$type> for $num {
            #[inline(always)]
            fn from(number: $type) -> Self {
                number.0 as Self
            }
        }
    )*};
}

pub use impl_from_integer_for_tuple;
