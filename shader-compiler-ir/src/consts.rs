// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::{
    prelude::*,
    text::{
        FromTextError, FromTextState, FromToTextListForm, IntegerSuffix, IntegerToken, Keyword,
        Punctuation, ToTextState, TokenKind,
    },
    BoolType, FloatType, IntegerType, PointerType, VectorType,
};
use alloc::vec::Vec;
use core::{
    convert::{TryFrom, TryInto},
    fmt,
};

/// a constant integer
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum ConstInteger {
    /// a constant 8-bit signed or unsigned integer
    Int8(u8),
    /// a constant 16-bit signed or unsigned integer
    Int16(u16),
    /// a constant 32-bit signed or unsigned integer
    Int32(u32),
    /// a constant 32-bit signed or unsigned integer of type `IntegerType::RelaxedInt32`
    RelaxedInt32(RelaxedInt32),
    /// a constant 64-bit signed or unsigned integer
    Int64(u64),
}

impl<'g> Internable<'g> for ConstInteger {
    type Interned = Const<'g>;
    fn intern(&self, global_state: &'g GlobalState<'g>) -> Interned<'g, Const<'g>> {
        Const::from(*self).intern(global_state)
    }
}

/// a constant 32-bit signed or unsigned integer of type `IntegerType::RelaxedInt32`
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct RelaxedInt32(pub u32);

impl From<u32> for RelaxedInt32 {
    fn from(v: u32) -> Self {
        RelaxedInt32(v)
    }
}

impl From<i32> for RelaxedInt32 {
    fn from(v: i32) -> Self {
        RelaxedInt32(v as u32)
    }
}

macro_rules! impl_const_types {
    (
        $name:ident {
            $(
                $enumerant:ident($type:ident $(as $as_type:ident)*),
            )+
        }
    ) => {
        $(
            impl From<$type> for $name {
                fn from(v: $type) -> Self {
                    $name::$enumerant(v $(as $as_type)*)
                }
            }

            impl From<$type> for Const<'_> {
                fn from(v: $type) -> Self {
                    $name::from(v).into()
                }
            }

            impl<'g> Internable<'g> for $type {
                type Interned = Const<'g>;
                fn intern(&self, global_state: &'g GlobalState<'g>) -> Interned<'g, Const<'g>> {
                    Const::from(*self).intern(global_state)
                }
            }
        )+
    };
}

impl_const_types! {
    ConstInteger {
        Int8(u8),
        Int8(i8 as u8),
        Int16(u16),
        Int16(i16 as u16),
        Int32(u32),
        Int32(i32 as u32),
        RelaxedInt32(RelaxedInt32),
        Int64(u64),
        Int64(i64 as u64),
    }
}

impl_const_types! {
    ConstFloat {
        Float16(Float16),
        Float32(Float32),
        RelaxedFloat32(RelaxedFloat32),
        Float64(Float64),
    }
}

impl From<ConstInteger> for Const<'_> {
    fn from(v: ConstInteger) -> Self {
        Const::Integer(v)
    }
}

/// there is no matching float size when bitcasting to/from integers
pub struct InvalidFloatSize;

impl ConstInteger {
    /// get `self`'s type
    pub fn get_type(self) -> IntegerType {
        match self {
            ConstInteger::Int8(_) => IntegerType::Int8,
            ConstInteger::Int16(_) => IntegerType::Int16,
            ConstInteger::Int32(_) => IntegerType::Int32,
            ConstInteger::RelaxedInt32(_) => IntegerType::RelaxedInt32,
            ConstInteger::Int64(_) => IntegerType::Int64,
        }
    }
}

/// a constant 16-bit float. The bits are stored as a `u16` in `Float16.0`.
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Float16(pub u16);

/// a constant 32-bit float. The bits are stored as a `u32` in `Float32.0`.
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Float32(pub u32);

/// a constant 32-bit float of type `FloatType::RelaxedFloat32`. The bits are stored as a `u32` in `RelaxedFloat32.0`.
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct RelaxedFloat32(pub u32);

impl From<Float32> for RelaxedFloat32 {
    fn from(v: Float32) -> Self {
        RelaxedFloat32(v.0)
    }
}

impl From<RelaxedFloat32> for Float32 {
    fn from(v: RelaxedFloat32) -> Self {
        Float32(v.0)
    }
}

/// a constant 64-bit float. The bits are stored as a `u64` in `Float64.0`.
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Float64(pub u64);

/// a constant float.
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum ConstFloat {
    /// a constant 16-bit float.
    Float16(Float16),
    /// a constant 32-bit float.
    Float32(Float32),
    /// a constant 32-bit float of type `FloatType::RelaxedFloat32`.
    RelaxedFloat32(RelaxedFloat32),
    /// a constant 64-bit float.
    Float64(Float64),
}

impl<'g> Internable<'g> for ConstFloat {
    type Interned = Const<'g>;
    fn intern(&self, global_state: &'g GlobalState<'g>) -> Interned<'g, Const<'g>> {
        Const::from(*self).intern(global_state)
    }
}

impl ConstFloat {
    /// get `self`'s type
    pub fn get_type(self) -> FloatType {
        match self {
            ConstFloat::Float16(_) => FloatType::Float16,
            ConstFloat::Float32(_) => FloatType::Float32,
            ConstFloat::RelaxedFloat32(_) => FloatType::RelaxedFloat32,
            ConstFloat::Float64(_) => FloatType::Float64,
        }
    }
}

impl From<ConstFloat> for Const<'_> {
    fn from(v: ConstFloat) -> Self {
        Const::Float(v)
    }
}

/// a constant non-scalable non-empty vector.
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct ConstVector<'g> {
    element_type: Interned<'g, Type<'g>>,
    elements: Vec<Interned<'g, Const<'g>>>,
}

impl<'g> ConstVector<'g> {
    /// create a new `ConstVector` using the provided elements.
    ///
    /// # Panics
    ///
    /// Panics if there are no provided elements.
    /// Panics if not all elements are the same type.
    pub fn new(
        elements: impl IntoIterator<Item = impl Internable<'g, Interned = Const<'g>>>,
        global_state: &'g GlobalState<'g>,
    ) -> Self {
        let elements: Vec<_> = elements
            .into_iter()
            .map(|v| v.intern(global_state))
            .collect();
        u32::try_from(elements.len()).expect("too many elements in ConstVector");
        let mut iter = elements.iter();
        let element_type = iter
            .next()
            .expect("vector must have non-zero size")
            .get()
            .get_type(global_state);
        for element in iter {
            assert_eq!(
                element.get().get_type(global_state),
                element_type,
                "vector must have consistent type"
            );
        }
        ConstVector {
            element_type,
            elements,
        }
    }
    /// get the type of an element.
    pub fn element_type(&self) -> Interned<'g, Type<'g>> {
        self.element_type
    }
    /// get the elements.
    pub fn elements(&self) -> &[Interned<'g, Const<'g>>] {
        &self.elements
    }
    /// get `self`'s type
    pub fn get_type(&self, global_state: &'g GlobalState<'g>) -> Interned<'g, Type> {
        VectorType {
            element: self.element_type,
            scalable: false,
            len: self
                .elements
                .len()
                .try_into()
                .expect("too many elements in ConstVector"),
        }
        .intern(global_state)
    }
}

impl<'g> Internable<'g> for ConstVector<'g> {
    type Interned = Const<'g>;
    fn intern(&self, global_state: &'g GlobalState<'g>) -> Interned<'g, Const<'g>> {
        Const::from(self.clone()).intern(global_state)
    }
}

impl<'g> From<ConstVector<'g>> for Const<'g> {
    fn from(v: ConstVector<'g>) -> Self {
        Const::Vector(v)
    }
}

/// a constant.
#[derive(Clone, Eq, PartialEq, Hash)]
pub enum Const<'g> {
    /// a constant integer
    Integer(ConstInteger),
    /// a constant float
    Float(ConstFloat),
    /// a constant boolean
    Bool(bool),
    /// a constant vector
    Vector(ConstVector<'g>),
    // FIXME: add scalable vectors
    /// a `undef` constant
    Undef(Interned<'g, Type<'g>>),
    /// a null pointer constant
    Null(PointerType<'g>),
    /// a function
    Function(FunctionRef<'g>),
}

impl<'g> From<bool> for Const<'g> {
    fn from(v: bool) -> Self {
        Const::Bool(v)
    }
}

impl<'g> Internable<'g> for bool {
    type Interned = Const<'g>;
    fn intern(&self, global_state: &'g GlobalState<'g>) -> Interned<'g, Const<'g>> {
        Const::from(*self).intern(global_state)
    }
}

impl<'g> Internable<'g> for FunctionRef<'g> {
    type Interned = Const<'g>;
    fn intern(&self, global_state: &'g GlobalState<'g>) -> Interned<'g, Const<'g>> {
        Const::from(self.clone()).intern(global_state)
    }
}

impl<'g> From<FunctionRef<'g>> for Const<'g> {
    fn from(v: FunctionRef<'g>) -> Self {
        Const::Function(v)
    }
}

impl<'g> Const<'g> {
    /// get `self`'s type
    pub fn get_type(&self, global_state: &'g GlobalState<'g>) -> Interned<'g, Type> {
        match *self {
            Const::Integer(const_int) => const_int.get_type().intern(global_state),
            Const::Float(const_float) => const_float.get_type().intern(global_state),
            Const::Bool(_) => BoolType.intern(global_state),
            Const::Vector(ref const_vector) => const_vector.get_type(global_state),
            Const::Undef(retval) => retval,
            Const::Null(ref pointer_type) => pointer_type.intern(global_state),
            Const::Function(ref function) => function.function_type.intern(global_state),
        }
    }
}

impl FromToTextListForm for ConstInteger {}

impl<'g> FromText<'g> for ConstInteger {
    type Parsed = Self;
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError> {
        let IntegerToken { value, suffix } = match state.peek_token()?.kind.integer() {
            Some(v) => v,
            _ => state
                .error_at_peek_token("expected integer literal")?
                .into(),
        };
        let retval = match suffix {
            Some(IntegerSuffix::I8) => value.try_into().map(ConstInteger::Int8),
            Some(IntegerSuffix::I16) => value.try_into().map(ConstInteger::Int16),
            Some(IntegerSuffix::I32) => value.try_into().map(ConstInteger::Int32),
            Some(IntegerSuffix::RI32) => value.try_into().map(RelaxedInt32).map(ConstInteger::RelaxedInt32),
            Some(IntegerSuffix::I64) => Ok(value).map(ConstInteger::Int64),
            None => state
                .error_at_peek_token(
                    "integer literal must have type suffix (for example, use `23i32` rather than `23`)",
                )?
                .into(),
        };
        let retval = match retval {
            Ok(retval) => retval,
            Err(_) => state
                .error_at_peek_token("integer literal too big for type")?
                .into(),
        };
        state.parse_token()?;
        Ok(retval)
    }
}

impl_display_as_to_text!(ConstInteger);

impl<'g> ToText<'g> for ConstInteger {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        match *self {
            ConstInteger::Int8(v) => write!(state, "{:#X}i8", v),
            ConstInteger::Int16(v) => write!(state, "{:#X}i16", v),
            ConstInteger::Int32(v) => write!(state, "{:#X}i32", v),
            ConstInteger::RelaxedInt32(RelaxedInt32(v)) => write!(state, "{:#X}ri32", v),
            ConstInteger::Int64(v) => write!(state, "{:#X}i64", v),
        }
    }
}

impl FromToTextListForm for ConstFloat {}

impl<'g> FromText<'g> for ConstFloat {
    type Parsed = Self;
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError> {
        let float_type = FloatType::from_text(state)?;
        let IntegerToken { value, suffix } = match state.peek_token()?.kind.integer() {
            Some(v) => v,
            _ => state
                .error_at_peek_token("expected integer literal")?
                .into(),
        };
        if suffix != None {
            state.error_at_peek_token("integer literal must not have suffix")?;
        }
        let retval = match float_type {
            FloatType::Float16 => value.try_into().map(Float16).map(Into::into),
            FloatType::Float32 => value.try_into().map(Float32).map(Into::into),
            FloatType::RelaxedFloat32 => value.try_into().map(RelaxedFloat32).map(Into::into),
            FloatType::Float64 => Ok(value).map(Float64).map(Into::into),
        };
        let retval = match retval {
            Ok(retval) => retval,
            Err(_) => state
                .error_at_peek_token("integer literal too big for type")?
                .into(),
        };
        state.parse_token()?;
        Ok(retval)
    }
}

impl_display_as_to_text!(ConstFloat);

impl FromToTextListForm for Float16 {}

impl<'g> ToText<'g> for Float16 {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        write!(state, "f16 {:#X}", self.0)
    }
}

impl FromToTextListForm for Float32 {}

impl<'g> ToText<'g> for Float32 {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        write!(state, "f32 {:#X}", self.0)
    }
}

impl FromToTextListForm for RelaxedFloat32 {}

impl<'g> ToText<'g> for RelaxedFloat32 {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        write!(state, "rf32 {:#X}", self.0)
    }
}

impl FromToTextListForm for Float64 {}

impl<'g> ToText<'g> for Float64 {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        write!(state, "f64 {:#X}", self.0)
    }
}

impl<'g> ToText<'g> for ConstFloat {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        match self {
            ConstFloat::Float16(v) => v.to_text(state),
            ConstFloat::Float32(v) => v.to_text(state),
            ConstFloat::RelaxedFloat32(v) => v.to_text(state),
            ConstFloat::Float64(v) => v.to_text(state),
        }
    }
}

impl FromToTextListForm for bool {}

impl<'g> FromText<'g> for bool {
    type Parsed = Self;
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError> {
        let retval = match state.peek_token()?.kind.keyword() {
            Some(Keyword::False) => false,
            Some(Keyword::True) => true,
            _ => state.error_at_peek_token("expected bool literal")?.into(),
        };
        state.parse_token()?;
        Ok(retval)
    }
}

impl<'g> ToText<'g> for bool {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        if *self {
            write!(state, "true")
        } else {
            write!(state, "false")
        }
    }
}

impl FromToTextListForm for ConstVector<'_> {}

impl<'g> FromText<'g> for ConstVector<'g> {
    type Parsed = Self;
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError> {
        state.parse_parenthesized(
            Punctuation::LessThan,
            "missing vector constant",
            Punctuation::GreaterThan,
            "missing closing angle bracket ('>')",
            |state| -> Result<Self, FromTextError> {
                let element = Const::from_text(state)?;
                let element_type = element.get().get_type(state.global_state());
                let mut elements = vec![element];
                while state.peek_token()?.kind.punct() == Some(Punctuation::Comma) {
                    state.parse_token()?;
                    let element_location = state.peek_token()?.span;
                    let element = Const::from_text(state)?;
                    if element.get().get_type(state.global_state()) != element_type {
                        state.error_at(element_location, "vector must have consistent type")?;
                    }
                    elements.push(element);
                }
                Ok(ConstVector::new(elements, state.global_state()))
            },
        )
    }
}

impl_display_as_to_text!(<'g> ConstVector<'g>);

impl<'g> ToText<'g> for ConstVector<'g> {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        let mut iter = self.elements.iter().copied();
        write!(state, "<")?;
        let first = iter.next().expect("vector must have non-zero size");
        first.to_text(state)?;
        for element in iter {
            write!(state, ", ")?;
            element.to_text(state)?;
        }
        write!(state, ">")
    }
}

impl FromToTextListForm for Const<'_> {}

impl<'g> FromText<'g> for Const<'g> {
    type Parsed = Interned<'g, Const<'g>>;
    fn from_text(
        state: &mut FromTextState<'g, '_>,
    ) -> Result<Interned<'g, Const<'g>>, FromTextError> {
        let retval = match state.peek_token()?.kind {
            TokenKind::Integer(_) => Const::Integer(ConstInteger::from_text(state)?),
            TokenKind::Keyword(Keyword::F16)
            | TokenKind::Keyword(Keyword::F32)
            | TokenKind::Keyword(Keyword::RF32)
            | TokenKind::Keyword(Keyword::F64) => Const::Float(ConstFloat::from_text(state)?),
            TokenKind::Keyword(Keyword::False) | TokenKind::Keyword(Keyword::True) => {
                Const::Bool(bool::from_text(state)?)
            }
            TokenKind::Punct(Punctuation::LessThan) => {
                Const::Vector(ConstVector::from_text(state)?)
            }
            TokenKind::Keyword(Keyword::Undef) => {
                state.parse_token()?;
                Const::Undef(Type::from_text(state)?)
            }
            TokenKind::Keyword(Keyword::Null) => {
                state.parse_token()?;
                Const::Null(PointerType::from_text(state)?)
            }
            TokenKind::Keyword(Keyword::Fn) => {
                state.parse_token()?;
                Const::Function(FunctionRef::from_text(state)?)
            }
            // FIXME: add scalable vectors
            _ => state.error_at_peek_token("missing constant")?.into(),
        };
        Ok(retval.intern(state.global_state()))
    }
}

impl_display_as_to_text!(<'g> Const<'g>);

impl<'g> ToText<'g> for Const<'g> {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        match self {
            Const::Integer(v) => v.to_text(state),
            Const::Float(v) => v.to_text(state),
            Const::Bool(v) => v.to_text(state),
            Const::Vector(v) => v.to_text(state),
            Const::Undef(ty) => {
                write!(state, "undef ")?;
                ty.to_text(state)
            }
            Const::Null(pointer_type) => {
                write!(state, "null ")?;
                pointer_type.to_text(state)
            }
            Const::Function(function) => {
                write!(state, "fn ")?;
                function.to_text(state)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DataPointerType, FunctionPointerType};
    use alloc::string::ToString;

    #[test]
    fn test_const_from_to_text() {
        #![allow(clippy::cognitive_complexity)]
        let global_state = GlobalState::new();
        macro_rules! test_const {
            ($global_state:ident, $text:literal, $const:expr, $formatted_text:literal) => {
                let parsed_const = Const::parse("", $text, &$global_state).unwrap();
                let expected_const = $const.intern(&$global_state);
                assert_eq!(parsed_const, expected_const);
                let text = expected_const.display().to_string();
                assert_eq!($formatted_text, text);
            };
            ($global_state:ident, $text:literal, $const:expr) => {
                test_const!($global_state, $text, $const, $text);
            };
        }
        test_const!(global_state, "0i8", 0u8, "0x0i8");
        test_const!(global_state, "10i8", 10u8, "0xAi8");
        test_const!(global_state, "0b10i8", 2u8, "0x2i8");
        test_const!(global_state, "0B10i8", 2u8, "0x2i8");
        test_const!(global_state, "0o10i8", 8u8, "0x8i8");
        test_const!(global_state, "0O10i8", 8u8, "0x8i8");
        test_const!(global_state, "0X10i8", 0x10u8, "0x10i8");
        test_const!(global_state, "0xFFi8", 0xFFu8);
        test_const!(global_state, "0x0i8", 0u8);
        test_const!(global_state, "0xFFFFi16", 0xFFFFu16);
        test_const!(global_state, "0xFFFFFFFFi32", 0xFFFF_FFFFu32);
        test_const!(global_state, "0xFFFFFFFFri32", RelaxedInt32(0xFFFF_FFFFu32));
        test_const!(
            global_state,
            "0xFFFFFFFFFFFFFFFFi64",
            0xFFFF_FFFF_FFFF_FFFFu64
        );
        test_const!(global_state, "f16 0xF000", Float16(0xF000));
        test_const!(global_state, "f32 0xFF000000", Float32(0xFF00_0000));
        test_const!(global_state, "rf32 0xFF000000", RelaxedFloat32(0xFF00_0000));
        test_const!(
            global_state,
            "f64 0xFF00000000000000",
            Float64(0xFF00_0000_0000_0000)
        );
        test_const!(
            global_state,
            "<0x1i8>",
            ConstVector::new(&[1u8], &global_state)
        );
        test_const!(
            global_state,
            "<0x1i8, 0x2i8>",
            ConstVector::new(&[1u8, 2u8], &global_state)
        );
        test_const!(
            global_state,
            "<0x1i8, 0x2i8, 0x3i8, 0x4i8>",
            ConstVector::new(&[1u8, 2, 3, 4], &global_state)
        );
        test_const!(
            global_state,
            "undef i8",
            IntegerType::Int8.intern(&global_state).undef()
        );
        test_const!(
            global_state,
            "undef data_ptr",
            DataPointerType.intern(&global_state).undef()
        );
        test_const!(global_state, "null data_ptr", DataPointerType.null());
        test_const!(
            global_state,
            "null fn[] -> !",
            FunctionPointerType {
                arguments: vec![],
                returns: Uninhabited
            }
            .null()
        );
        // TODO: test Const::Function
    }
}
