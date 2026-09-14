macro_rules! impl_string_type {
    ( $( $(#[$outer:meta])* pub struct $name:ident; )* ) => {
        $(
            $(#[$outer])*
            #[derive(Debug, Clone, PartialEq, Eq, Hash)]
            #[cfg_attr(feature = "codec", derive(serde::Serialize, serde::Deserialize))]
            pub struct $name(String);

            impl<T: Into<String>> From<T> for $name {
                fn from(value: T) -> Self {
                    Self(value.into())
                }
            }

            impl std::fmt::Display for $name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    f.write_str(&self.0)
                }
            }

            impl $name {
                pub fn new(s: impl Into<String>) -> Self {
                    Self(s.into())
                }
            }
        )*
    };
}

impl_string_type! {
    /// Import name element of AST
    pub struct ImportName;

    /// `ValueName` value name element of AST. It's basic entity for:
    /// - `Struct` type declaration
    /// - `LetBinding` declaration
    /// - `Binding` declaration
    /// - `ExpressionValue` declaration
    pub struct ValueName;

    /// Inner value name type
    pub struct InnerValueName;

    /// Label name type
    pub struct LabelName;

    /// Function name type
    pub struct FunctionName;

    /// Constant name type
    pub struct ConstantName;

    /// Type name representation
    pub struct TypeName;
}

impl From<ValueName> for InnerValueName {
    fn from(value: ValueName) -> Self {
        Self(value.0)
    }
}

impl From<ValueName> for ConstantName {
    fn from(value: ValueName) -> Self {
        Self(value.0)
    }
}
