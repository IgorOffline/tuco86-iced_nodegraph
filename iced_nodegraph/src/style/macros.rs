//! Declarative macro pairing a resolved `Style` struct with its `Option`-wrapped
//! `Config` overlay from a single field list.
//!
//! A `Style` holds concrete values used at render time; a `Config` is a partial
//! overlay (`Option` per field) that the user merges (default + overrides) and
//! resolves against a base style. The two structs and the glue between them
//! (`merge`, `has_overrides`, `resolve`, `From<Style>`) enumerate the same field
//! list, so keeping them in sync by hand is redundant and drift-prone. This
//! macro owns that enumeration; ergonomic setters, presets, `Default` and theme
//! constructors stay hand-written in separate `impl` blocks.
//!
//! Per-field kind:
//! - `value  name: T`      -> Style `T`,         Config `Option<T>`
//! - `option name: T`      -> Style `Option<T>`, Config `Option<T>`
//! - `nested name: T => C` -> Style `Option<T>`, Config `Option<C>`
//!   (`C: Default`, `C::resolve(&self) -> Option<T>`, and `C: From<T>`)

/// Style-side field type for a field kind.
macro_rules! sc_style_ty {
    (value $t:ty) => { $t };
    (option $t:ty) => { ::core::option::Option<$t> };
    (nested $t:ty => $c:ty) => { ::core::option::Option<$t> };
}

/// Config-side field type for a field kind.
macro_rules! sc_config_ty {
    (value $t:ty) => { ::core::option::Option<$t> };
    (option $t:ty) => { ::core::option::Option<$t> };
    (nested $t:ty => $c:ty) => { ::core::option::Option<$c> };
}

/// Resolve one field against the base style. `$sv` is `self.field`, `$bv` is
/// `base.field`.
macro_rules! sc_resolve {
    (value, $sv:expr, $bv:expr) => {
        $sv.clone().unwrap_or_else(|| $bv.clone())
    };
    (option, $sv:expr, $bv:expr) => {
        $sv.clone().or_else(|| $bv.clone())
    };
    (nested, $sv:expr, $bv:expr) => {
        match &$sv {
            ::core::option::Option::Some(c) => c.resolve(),
            ::core::option::Option::None => $bv.clone(),
        }
    };
}

/// Build one config field from the corresponding resolved style field `$sv`
/// (`style.field`).
macro_rules! sc_from {
    (value, $sv:expr) => {
        ::core::option::Option::Some($sv.clone())
    };
    (option, $sv:expr) => {
        $sv.clone()
    };
    (nested, $sv:expr) => {
        $sv.clone().map(::core::convert::Into::into)
    };
}

/// Generate a `Style`/`Config` pair plus their mechanical glue from one field
/// list. See the module docs for the field-kind grammar.
macro_rules! style_config {
    (
        $(#[$smeta:meta])*
        struct $Style:ident ;
        $(#[$cmeta:meta])*
        struct $Config:ident ;
        fields {
            $( $(#[$fmeta:meta])* $kind:ident $name:ident : $t:ty $(=> $c:ty)? ),* $(,)?
        }
    ) => {
        $(#[$smeta])*
        #[derive(Debug, Clone, PartialEq)]
        pub struct $Style {
            $( $(#[$fmeta])* pub $name: sc_style_ty!($kind $t $(=> $c)?), )*
        }

        $(#[$cmeta])*
        #[derive(Debug, Clone, Default, PartialEq)]
        pub struct $Config {
            $( $(#[$fmeta])* pub $name: sc_config_ty!($kind $t $(=> $c)?), )*
        }

        impl $Config {
            /// Merges two configs. `self` takes priority; `other` fills the gaps.
            pub fn merge(&self, other: &Self) -> Self {
                Self {
                    $( $name: self.$name.clone().or_else(|| other.$name.clone()), )*
                }
            }

            /// Returns `true` if any field overrides the resolved base.
            pub fn has_overrides(&self) -> bool {
                false $( || self.$name.is_some() )*
            }

            /// Resolves to a concrete style, filling unset fields from `base`.
            pub fn resolve(&self, base: &$Style) -> $Style {
                $Style {
                    $( $name: sc_resolve!($kind, self.$name, base.$name), )*
                }
            }
        }

        impl ::core::convert::From<$Style> for $Config {
            fn from(style: $Style) -> Self {
                Self {
                    $( $name: sc_from!($kind, style.$name), )*
                }
            }
        }
    };
}
