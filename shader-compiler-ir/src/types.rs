// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::prelude::*;
use crate::text::FromTextError;
use crate::text::FromTextState;
use crate::text::IntegerToken;
use crate::text::Keyword;
use crate::text::Punctuation;
use crate::text::ToTextState;
use crate::text::TokenKind;
use std::convert::TryInto;
use std::fmt;
use std::ops::Deref;
use std::ops::DerefMut;

pub trait GenericType<'g>: Internable<'g, Interned = Type<'g>> {
    fn undef(&self, global_state: &'g GlobalState<'g>) -> Const<'g> {
        self.intern(global_state).undef()
    }
    fn pointer(&self, global_state: &'g GlobalState<'g>) -> PointerType<'g> {
        self.intern(global_state).pointer()
    }
    fn new_value_definition<Name: Internable<'g, Interned = str>>(
        &self,
        name: Name,
        global_state: &'g GlobalState<'g>,
    ) -> ValueDefinition<'g> {
        ValueDefinition::new(self, name, global_state)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum IntegerType {
    Int8,
    Int16,
    Int32,
    Int64,
}

impl<'g> Internable<'g> for IntegerType {
    type Interned = Type<'g>;
    fn intern(&self, global_state: &'g GlobalState<'g>) -> Interned<'g, Type<'g>> {
        Type::from(*self).intern(global_state)
    }
}

impl<'g> GenericType<'g> for IntegerType {}

impl From<IntegerType> for Type<'_> {
    fn from(v: IntegerType) -> Self {
        Type::Integer(v)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum FloatType {
    Float16,
    Float32,
    Float64,
}

impl<'g> Internable<'g> for FloatType {
    type Interned = Type<'g>;
    fn intern(&self, global_state: &'g GlobalState<'g>) -> Interned<'g, Type<'g>> {
        Type::from(*self).intern(global_state)
    }
}

impl<'g> GenericType<'g> for FloatType {}

impl From<FloatType> for Type<'_> {
    fn from(v: FloatType) -> Self {
        Type::Float(v)
    }
}

mod private {
    #[doc(hidden)]
    #[derive(Clone, Eq, PartialEq, Hash, Debug)]
    pub enum Void {}
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum OpaqueType<'g> {
    // TODO: implement
    #[doc(hidden)]
    _Unimplemented(&'g (), private::Void),
}

impl<'g> Internable<'g> for OpaqueType<'g> {
    type Interned = Type<'g>;
    fn intern(&self, _global_state: &'g GlobalState<'g>) -> Interned<'g, Type<'g>> {
        match self {
            OpaqueType::_Unimplemented(_, v) => match *v {},
        }
    }
}

impl<'g> GenericType<'g> for OpaqueType<'g> {}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct BoolType;

impl<'g> Internable<'g> for BoolType {
    type Interned = Type<'g>;
    fn intern(&self, global_state: &'g GlobalState<'g>) -> Interned<'g, Type<'g>> {
        Type::from(*self).intern(global_state)
    }
}

impl<'g> GenericType<'g> for BoolType {}

impl From<BoolType> for Type<'_> {
    fn from(v: BoolType) -> Self {
        Type::Bool(v)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct PointerType<'g> {
    pub pointee: Interned<'g, Type<'g>>,
}

impl<'g> Internable<'g> for PointerType<'g> {
    type Interned = Type<'g>;
    fn intern(&self, global_state: &'g GlobalState<'g>) -> Interned<'g, Type<'g>> {
        Type::from(*self).intern(global_state)
    }
}

impl<'g> GenericType<'g> for PointerType<'g> {}

impl<'g> PointerType<'g> {
    pub fn null(self) -> Const<'g> {
        Const::Null(self)
    }
}

impl<'g> From<PointerType<'g>> for Type<'g> {
    fn from(v: PointerType<'g>) -> Self {
        Type::Pointer(v)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct VectorType<'g> {
    pub len: usize,
    pub scalable: bool,
    pub element: Interned<'g, Type<'g>>,
}

impl<'g> Internable<'g> for VectorType<'g> {
    type Interned = Type<'g>;
    fn intern(&self, global_state: &'g GlobalState<'g>) -> Interned<'g, Type<'g>> {
        Type::from(*self).intern(global_state)
    }
}

impl<'g> GenericType<'g> for VectorType<'g> {}

impl<'g> From<VectorType<'g>> for Type<'g> {
    fn from(v: VectorType<'g>) -> Self {
        Type::Vector(v)
    }
}

impl<'g> From<OpaqueType<'g>> for Type<'g> {
    fn from(v: OpaqueType<'g>) -> Self {
        Type::Opaque(v)
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum Type<'g> {
    Integer(IntegerType),
    Float(FloatType),
    Bool(BoolType),
    Pointer(PointerType<'g>),
    Vector(VectorType<'g>),
    Opaque(OpaqueType<'g>),
}

impl<'g> Internable<'g> for Type<'g> {
    type Interned = Type<'g>;
    fn intern(&self, global_state: &'g GlobalState<'g>) -> Interned<'g, Type<'g>> {
        global_state.intern(self)
    }
}

impl<'g> GenericType<'g> for Type<'g> {}

impl<'g> GenericType<'g> for Interned<'g, Type<'g>> {}

/// if a type or value `T` is inhabited (is reachable)
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Inhabitable<T> {
    /// type or value `T` is inhabited (is reachable)
    Inhabited(T),
    /// uninhabited (unreachable)
    Uninhabited,
}

pub use Inhabitable::*;

impl<T> Inhabitable<T> {
    /// like `Option::as_ref`
    pub fn as_ref(&self) -> Inhabitable<&T> {
        match self {
            Inhabited(v) => Inhabited(v),
            Uninhabited => Uninhabited,
        }
    }
    /// like `Option::as_mut`
    pub fn as_mut(&mut self) -> Inhabitable<&mut T> {
        match self {
            Inhabited(v) => Inhabited(v),
            Uninhabited => Uninhabited,
        }
    }
    /// like `Option::map`
    pub fn map<F: FnOnce(T) -> R, R>(self, f: F) -> Inhabitable<R> {
        match self {
            Inhabited(v) => Inhabited(f(v)),
            Uninhabited => Uninhabited,
        }
    }
    /// like `Option::as_deref`
    pub fn as_deref(&self) -> Inhabitable<&T::Target>
    where
        T: Deref,
    {
        self.as_ref().map(|v| &**v)
    }
    /// like `Option::as_deref_mut`
    pub fn as_deref_mut(&mut self) -> Inhabitable<&mut T::Target>
    where
        T: DerefMut,
    {
        self.as_mut().map(|v| &mut **v)
    }
    /// return `Some` if `self` is `Inhabited`
    pub fn inhabited(self) -> Option<T> {
        match self {
            Inhabited(v) => Some(v),
            Uninhabited => None,
        }
    }
}

macro_rules! impl_from_to_text_for_keyword_type {
    ($type:ident {
        $($kw:ident => $value:path,)+
        _ => $error_msg:expr,
    }) => {
        impl<'g> FromText<'g> for $type {
            type Parsed = Self;
            fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError> {
                let retval = match state.peek_token()?.kind {
                    $(TokenKind::Keyword(Keyword::$kw) => $value,)+
                    _ => state.error_at_peek_token($error_msg)?.into(),
                };
                state.parse_token()?;
                Ok(retval)
            }
        }

        impl<'g> ToText<'g> for $type {
            fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
                match self {
                    $(
                        $value => write!(state, "{}", Keyword::$kw),
                    )+
                }
            }
        }
    };
}

impl_from_to_text_for_keyword_type! {
    IntegerType {
        I8 => IntegerType::Int8,
        I16 => IntegerType::Int16,
        I32 => IntegerType::Int32,
        I64 => IntegerType::Int64,
        _ => "invalid integer type",
    }
}

impl_from_to_text_for_keyword_type! {
    FloatType {
        F16 => FloatType::Float16,
        F32 => FloatType::Float32,
        F64 => FloatType::Float64,
        _ => "invalid float type",
    }
}

impl_from_to_text_for_keyword_type! {
    BoolType {
        Bool => BoolType,
        _ => "invalid bool type",
    }
}

impl<'g> FromText<'g> for VectorType<'g> {
    type Parsed = Self;
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError> {
        state.parse_parenthesized(
            Punctuation::LessThan,
            "missing opening angle bracket: Punctuation::LessThan",
            Punctuation::GreaterThan,
            "missing closing angle bracket: '>'",
            |state| -> Result<VectorType<'g>, FromTextError> {
                let scalable = if let TokenKind::Keyword(Keyword::VScale) = state.peek_token()?.kind
                {
                    state.parse_token()?;
                    state.parse_keyword_token_or_error(Keyword::X, "missing x after vscale")?;
                    true
                } else {
                    false
                };
                let len = state.parse_token()?;
                let len: usize = match len.kind {
                    TokenKind::Integer(IntegerToken { value, suffix }) => {
                        if suffix.is_some() {
                            state.error_at(
                                len.span,
                                "vector length value must not have type suffix",
                            )?;
                        }
                        match value.try_into() {
                            Ok(len) => len,
                            Err(_) => state
                                .error_at(len.span, "vector length value too big")?
                                .into(),
                        }
                    }
                    _ => state
                        .error_at(len.span, "missing vector length value")?
                        .into(),
                };
                state.parse_keyword_token_or_error(Keyword::X, "missing x after vscale")?;
                Ok(VectorType {
                    len,
                    scalable,
                    element: Type::from_text(state)?,
                })
            },
        )
    }
}

impl<'g> ToText<'g> for VectorType<'g> {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        let VectorType {
            len,
            scalable,
            element,
        } = *self;
        write!(state, "<")?;
        if scalable {
            write!(state, "vscale x ")?;
        }
        write!(state, "{} x ", len)?;
        element.to_text(state)?;
        write!(state, ">")
    }
}

impl<'g> FromText<'g> for PointerType<'g> {
    type Parsed = Self;
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError> {
        state.parse_punct_token_or_error(Punctuation::Asterisk, "expected pointer type")?;
        Ok(PointerType {
            pointee: Type::from_text(state)?,
        })
    }
}

impl<'g> ToText<'g> for PointerType<'g> {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        write!(state, "*")?;
        self.pointee.to_text(state)
    }
}

impl<'g> FromText<'g> for OpaqueType<'g> {
    type Parsed = Self;
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError> {
        // TODO: implement
        state
            .error_at_peek_token("OpaqueType can't be parsed")?
            .into()
    }
}

impl<'g> ToText<'g> for OpaqueType<'g> {
    fn to_text(&self, _state: &mut ToTextState<'g, '_>) -> fmt::Result {
        match self {
            OpaqueType::_Unimplemented(_, v) => match *v {},
        }
    }
}

impl<'g> FromText<'g> for Type<'g> {
    type Parsed = Interned<'g, Type<'g>>;
    fn from_text(
        state: &mut FromTextState<'g, '_>,
    ) -> Result<Interned<'g, Type<'g>>, FromTextError> {
        let retval = match state.peek_token()?.kind {
            TokenKind::Keyword(Keyword::I8)
            | TokenKind::Keyword(Keyword::I16)
            | TokenKind::Keyword(Keyword::I32)
            | TokenKind::Keyword(Keyword::I64) => Type::Integer(IntegerType::from_text(state)?),
            TokenKind::Keyword(Keyword::F16)
            | TokenKind::Keyword(Keyword::F32)
            | TokenKind::Keyword(Keyword::F64) => Type::Float(FloatType::from_text(state)?),
            TokenKind::Keyword(Keyword::Bool) => Type::Bool(BoolType::from_text(state)?),
            TokenKind::Punct(Punctuation::LParen) => {
                return state.parse_parenthesized(
                    Punctuation::LParen,
                    "",
                    Punctuation::RParen,
                    "missing closing parenthesis: ')'",
                    Type::from_text,
                );
            }
            TokenKind::Punct(Punctuation::LessThan) => Type::Vector(VectorType::from_text(state)?),
            TokenKind::Punct(Punctuation::Asterisk) => {
                Type::Pointer(PointerType::from_text(state)?)
            }
            // TODO: add OpaqueType
            _ => state.error_at_peek_token("expected type")?.into(),
        };
        Ok(state.global_state().intern(&retval))
    }
}

impl<'g> ToText<'g> for Type<'g> {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        match self {
            Type::Integer(v) => v.to_text(state),
            Type::Float(v) => v.to_text(state),
            Type::Bool(v) => v.to_text(state),
            Type::Pointer(v) => v.to_text(state),
            Type::Vector(v) => v.to_text(state),
            Type::Opaque(v) => v.to_text(state),
        }
    }
}

impl<'g> Interned<'g, Type<'g>> {
    pub fn pointer(self) -> PointerType<'g> {
        PointerType { pointee: self }
    }
    pub fn undef(self) -> Const<'g> {
        Const::Undef(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_from_to_text() {
        let global_state = GlobalState::new();
        macro_rules! test_type {
            ($global_state:ident, $text:literal, $type:expr, $formatted_text:literal) => {
                let parsed_type = Type::parse("", $text, &$global_state).unwrap();
                let expected_type = $type.intern(&$global_state);
                assert_eq!(parsed_type, expected_type);
                let text = expected_type.display().to_string();
                assert_eq!($formatted_text, text);
            };
            ($global_state:ident, $text:literal, $type:expr) => {
                test_type!($global_state, $text, $type, $text);
            };
        }
        test_type!(global_state, "i8", IntegerType::Int8);
        test_type!(global_state, "i16", IntegerType::Int16);
        test_type!(global_state, "i32", IntegerType::Int32);
        test_type!(global_state, "i64", IntegerType::Int64);
        test_type!(global_state, "f16", FloatType::Float16);
        test_type!(global_state, "f32", FloatType::Float32);
        test_type!(global_state, "f64", FloatType::Float64);
        test_type!(global_state, "bool", BoolType);
        test_type!(
            global_state,
            "*i8",
            IntegerType::Int8.intern(&global_state).pointer()
        );
        test_type!(
            global_state,
            "<4 x f16>",
            VectorType {
                len: 4,
                scalable: false,
                element: FloatType::Float16.intern(&global_state)
            }
        );
        test_type!(
            global_state,
            "<vscale x 7 x f32>",
            VectorType {
                len: 7,
                scalable: true,
                element: FloatType::Float32.intern(&global_state)
            }
        );
        test_type!(
            global_state,
            "(<vscale x 7 x ((* bool))>)",
            VectorType {
                len: 7,
                scalable: true,
                element: BoolType
                    .intern(&global_state)
                    .pointer()
                    .intern(&global_state)
            },
            "<vscale x 7 x *bool>"
        );
        // FIXME: add tests for opaque types
    }
}
