// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

//! conversion from/to text

use crate::{prelude::*, IdRef, StructType};
use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use arrayvec::{Array, ArrayVec};
use core::{
    borrow::{Borrow, BorrowMut},
    fmt,
    marker::PhantomData,
    mem,
    ops::Range,
    str::FromStr,
};
use hashbrown::{hash_map::Entry, HashMap};
use once_cell::unsync::OnceCell;
use unicode_width::UnicodeWidthChar;

macro_rules! impl_display_as_to_text {
    (<$g:lifetime> $ty:ty) => {
        impl<$g> core::fmt::Display for $ty {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                core::fmt::Display::fmt(&<Self as $crate::text::ToText<$g>>::display(self), f)
            }
        }
        impl<$g> core::fmt::Debug for $ty {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                core::fmt::Debug::fmt(&<Self as $crate::text::ToText<$g>>::display(self), f)
            }
        }
    };
    ($ty:ty) => {
        impl_display_as_to_text!(<'g> $ty);
    };
}

macro_rules! impl_struct_with_default_from_to_text {
    (
        $(#[doc = $struct_doc:expr])*
        #[name_keyword = $name_keyword:ident]
        #[from_text($state:ident <$g:lifetime> $parsed_type:ty, $retval:ident => $into_parsed_type:expr)]
        $(#[$struct_meta:meta])*
        $vis:vis struct $struct_name:ident {
            $(
                $(#[doc = $doc:expr])+
                $member:ident:$member_ty:ty = $member_init:expr,
            )+
        }
    ) => {
        $(#[doc = $struct_doc])*
        #[derive(Clone, Eq, PartialEq, Hash)]
        $vis struct $struct_name {
            $(
                $(#[doc = $doc])+
                $vis $member: $member_ty,
            )+
        }

        impl_display_as_to_text!($struct_name);

        impl Default for $struct_name {
            fn default() -> Self {
                $struct_name {
                    $(
                        $member: $member_init,
                    )+
                }
            }
        }

        impl crate::text::FromToTextListForm for $struct_name {
            fn from_to_text_list_form() -> crate::text::ListForm {
                crate::text::ListForm::STATEMENTS
            }
        }

        impl<'g> crate::text::FromText<'g> for $struct_name {
            type Parsed = $parsed_type;
            fn from_text($state: &mut crate::text::FromTextState<'g, '_>) -> Result<Self::Parsed, crate::text::FromTextError> {
                if let Some(stringify!($name_keyword)) = $state.peek_token()?.kind.raw_identifier() {
                    $state.parse_token()?;
                } else {
                    $state.error_at_peek_token(concat!("missing ", stringify!($name_keyword)))?;
                }
                $state.parse_parenthesized(
                    crate::text::Punctuation::LCurlyBrace,
                    "missing opening curly brace ('{')",
                    crate::text::Punctuation::RCurlyBrace,
                    "missing closing curly brace ('}')",
                    |$state| -> Result<Self::Parsed, crate::text::FromTextError> {
                        let mut $retval = Self::default();
                        struct Specified {
                            $($member: bool,)+
                        }
                        let mut specified = Specified {
                            $($member: false,)+
                        };
                        while let Some(ident) = $state.peek_token()?.kind.raw_identifier() {
                            match ident {
                                $(
                                    stringify!($member) => {
                                        if specified.$member {
                                            $state.error_at_peek_token(concat!("duplicate field: ", stringify!($member), " already specified"))?;
                                        }
                                        specified.$member = true;
                                        $state.parse_token()?;
                                        $state.parse_punct_token_or_error(crate::text::Punctuation::Colon, "missing colon (':') after field name")?;
                                        $retval.$member = <$member_ty>::from_text($state)?;
                                    }
                                )+
                                _ => $state.error_at_peek_token("unknown field name")?.into(),
                            }
                            $state.parse_punct_token_or_error(crate::text::Punctuation::Comma, "missing comma (',') after field value")?;
                        }
                        $into_parsed_type
                    },
                )
            }
        }

        impl<'g> crate::text::ToText<'g> for $struct_name {
            fn to_text(&self, state: &mut crate::text::ToTextState<'g, '_>) -> core::fmt::Result {
                writeln!(state, concat!(stringify!($name_keyword), " {{"))?;
                state.indent(|state| -> core::fmt::Result {
                    $(
                        write!(state, concat!(stringify!($member), ": "))?;
                        self.$member.to_text(state)?;
                        writeln!(state, ",")?;
                    )+
                    Ok(())
                })?;
                write!(state, "}}")
            }
        }
    };
}

/// the struct managing the source code for `FromText`.
#[derive(Debug)]
pub struct FromTextSourceCode<'a> {
    /// the file name for the source code
    pub file_name: &'a str,
    /// the source code
    pub text: &'a str,
    line_start_byte_indexes: OnceCell<Vec<usize>>,
}

impl<'a> FromTextSourceCode<'a> {
    /// create a new `FromTextSourceCode`
    pub fn new(file_name: &'a str, text: &'a str) -> Self {
        Self {
            file_name,
            text,
            line_start_byte_indexes: OnceCell::new(),
        }
    }
    /// byte indexes of line starts
    /// always starts with 0
    pub fn line_start_byte_indexes(&self) -> &[usize] {
        self.line_start_byte_indexes.get_or_init(|| {
            let mut line_start_byte_indexes = vec![0];
            for (index, byte) in self.text.bytes().enumerate() {
                if byte == b'\n' {
                    // don't need to specifically check for "\r\n" since
                    // line start still is right after '\n'
                    line_start_byte_indexes.push(index + 1);
                }
            }
            line_start_byte_indexes
        })
    }
    /// 0-based line number of the line containing byte_index
    pub fn line_index_of_containing_line(&self, byte_index: usize) -> usize {
        let line_start_byte_indexes = self.line_start_byte_indexes();
        match line_start_byte_indexes.binary_search(&byte_index) {
            Ok(index) => index,
            Err(index) => index - 1,
        }
    }
}

/// the location of an error produced by `FromText`
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct FromTextErrorLocation {
    /// the file name for the source code
    pub file_name: String,
    /// the 0-based index of the source code byte where the error occurred
    pub byte_index: usize,
    /// the 1-based line number where the error occurred
    pub line_number: usize,
    /// the 1-based column number where the error occurred
    pub column_number: usize,
}

impl fmt::Display for FromTextErrorLocation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}:{}:{}",
            self.file_name, self.line_number, self.column_number
        )
    }
}

/// an error produced by `FromText`
#[derive(Clone, Debug)]
pub struct FromTextError {
    /// the source location where the error occurred
    pub location: FromTextErrorLocation,
    /// the description of the error
    pub message: String,
}

impl fmt::Display for FromTextError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: error: {}", self.location, self.message)
    }
}

/// a location in the source code for `FromText`
#[derive(Copy, Clone, Debug)]
pub struct TextLocation<'a> {
    byte_index: usize,
    source_code: &'a FromTextSourceCode<'a>,
}

impl PartialEq for TextLocation<'_> {
    fn eq(&self, rhs: &Self) -> bool {
        assert_eq!(
            self.source_code as *const _, rhs.source_code,
            "can only compare TextLocation values within the same source"
        );
        self.byte_index == rhs.byte_index
    }
}

impl Eq for TextLocation<'_> {}

impl Iterator for TextLocation<'_> {
    type Item = char;
    fn next(&mut self) -> Option<char> {
        let mut chars = self.source_code.text[self.byte_index..].chars();
        let retval = chars.next()?;
        self.byte_index = self.source_code.text.len() - chars.as_str().len();
        Some(retval)
    }
}

impl<'a> TextLocation<'a> {
    /// create a new `TextLocation` at the specified 0-based byte index.
    ///
    /// # Panics
    ///
    /// Panics if byte_index is not on a `char` boundary (see `str::is_char_boundary()`)
    pub fn new(byte_index: usize, source_code: &'a FromTextSourceCode<'a>) -> Self {
        assert!(source_code.text.is_char_boundary(byte_index));
        Self {
            byte_index,
            source_code,
        }
    }
    /// return the next `char` in the source code without advancing `self`
    pub fn peek(&self) -> Option<char> {
        let mut copy = *self;
        copy.next()
    }
    /// get the source code
    pub fn source_code(&self) -> &'a FromTextSourceCode<'a> {
        self.source_code
    }
    /// get the 0-based byte index into the source code
    pub fn byte_index(&self) -> usize {
        self.byte_index
    }
}

impl From<TextLocation<'_>> for FromTextErrorLocation {
    /// Convert to `FromTextErrorLocation`.
    /// This is a relatively expensive operation since line and column information needs to be calculated.
    fn from(text_location: TextLocation) -> FromTextErrorLocation {
        let file_name = text_location.source_code.file_name.into();
        let byte_index = text_location.byte_index();
        let text = text_location.source_code.text;
        let line_index = text_location
            .source_code
            .line_index_of_containing_line(text_location.byte_index);
        let line_start_index = text_location.source_code.line_start_byte_indexes()[line_index];
        let line_number = line_index + 1;
        const TAB_WIDTH: usize = 4;
        let column_number = 1 + text[line_start_index..byte_index]
            .chars()
            .fold(0, |col, ch| {
                // col is zero-based
                if ch == '\t' {
                    (col + TAB_WIDTH) / TAB_WIDTH * TAB_WIDTH
                } else {
                    col + ch.width().unwrap_or(0)
                }
            });
        FromTextErrorLocation {
            file_name,
            byte_index,
            line_number,
            column_number,
        }
    }
}

/// a range of locations in source code -- for `FromText`
#[derive(Copy, Clone, Debug)]
pub struct TextSpan<'a> {
    start_byte_index: usize,
    end_byte_index: usize,
    source_code: &'a FromTextSourceCode<'a>,
}

impl Eq for TextSpan<'_> {}

impl PartialEq for TextSpan<'_> {
    fn eq(&self, rhs: &Self) -> bool {
        assert_eq!(
            self.source_code as *const _, rhs.source_code,
            "can only compare TextSpan values within the same source"
        );
        self.byte_indexes() == rhs.byte_indexes()
    }
}

impl<'a> TextSpan<'a> {
    /// create a new `TextSpan` starting with `start` and up to but not including `end`.
    ///
    /// # Panics
    ///
    /// Panics if `start` and `end` are not in the same source code.
    /// Panics if `start` comes after `end`.
    pub fn new(start: TextLocation<'a>, end: TextLocation<'a>) -> Self {
        assert_eq!(
            start.source_code as *const _, end.source_code,
            "TextSpan start and end must be within the same source"
        );
        assert!(
            start.byte_index <= end.byte_index,
            "TextSpan start must not come after end"
        );
        Self {
            start_byte_index: start.byte_index,
            end_byte_index: end.byte_index,
            source_code: start.source_code,
        }
    }
    /// gets the 0-based byte indexes as a `Range<usize>`
    pub fn byte_indexes(self) -> Range<usize> {
        self.start_byte_index..self.end_byte_index
    }
    /// gets the source code
    pub fn source_code(self) -> &'a FromTextSourceCode<'a> {
        self.source_code
    }
    /// gets the text contained in this span. The lifetime is that of the source code, not of `self`.
    pub fn text(self) -> &'a str {
        &self.source_code().text[self.byte_indexes()]
    }
    /// gets the length in bytes of the text contained in this span.
    pub fn len(self) -> usize {
        self.end_byte_index - self.start_byte_index
    }
    /// checks if this span is empty.
    pub fn is_empty(self) -> bool {
        self.len() == 0
    }
    /// gets the `TextLocation` for the start of this span.
    pub fn start(self) -> TextLocation<'a> {
        TextLocation::new(self.start_byte_index, self.source_code)
    }
    /// gets the `TextLocation` for the first character after the end of this span.
    pub fn end(self) -> TextLocation<'a> {
        TextLocation::new(self.end_byte_index, self.source_code)
    }
}

impl From<TextSpan<'_>> for FromTextErrorLocation {
    fn from(span: TextSpan) -> FromTextErrorLocation {
        span.start().into()
    }
}

mod private {
    pub trait Sealed {}
    impl Sealed for char {}
}

/// extension trait for `char` to provide `is_identifier_start` and `is_identifier_continue`
pub trait FromTextCharExt: Copy + private::Sealed {
    /// check if this is a valid `char` for starting an identifier.
    /// This is true only for `'_'` and ascii alphabetic (`is_ascii_alphabetic`) characters.
    /// This is a subset of the values accepted by `is_identifier_continue`.
    fn is_identifier_start(self) -> bool;
    /// check if this is a valid `char` for starting an identifier.
    /// This is true only for `'_'` and ascii alphanumeric (`is_ascii_alphanumeric`) characters.
    /// This is a superset of the values accepted by `is_identifier_start`.
    fn is_identifier_continue(self) -> bool;
}

impl FromTextCharExt for char {
    fn is_identifier_start(self) -> bool {
        self == '_' || self.is_ascii_alphabetic()
    }
    fn is_identifier_continue(self) -> bool {
        self.is_identifier_start() || self.is_ascii_digit()
    }
}

/// the error type returned when `FromStr::from_str` fails to match a keyword
#[derive(Debug)]
pub struct ParseKeywordError;

macro_rules! keywords {
    (
        $(#[doc = $keyword_enum_doc:literal])*
        $vis:vis enum $keyword_enum:ident {
            $(#[doc = $doc1:expr] $name1:ident = $text1:literal,)*
            $name2:ident = $text2:literal,
            $($(#[doc = $doc3:expr])* $name3:ident = $text3:literal,)*
        }
    ) => {
        keywords! {
            $(#[doc = $keyword_enum_doc])*
            $vis enum $keyword_enum {
                $(#[doc = $doc1] $name1 = $text1,)*
                #[doc = concat!("The keyword \"", $text2, "\"")]
                $name2 = $text2,
                $($(#[doc = $doc3])* $name3 = $text3,)*
            }
        }
    };
    (
        $(#[doc = $keyword_enum_doc:literal])*
        $vis:vis enum $keyword_enum:ident {
            $(#[doc = $doc:expr] $name:ident = $text:literal,)+
        }
    ) => {
        $(#[doc = $keyword_enum_doc])*
        #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
        $vis enum $keyword_enum {
            $(
                #[doc = $doc]
                $name,
            )+
        }

        impl $keyword_enum {
            /// Get the textual form of `self`
            pub fn text(self) -> &'static str {
                match self {
                    $(
                        $keyword_enum::$name => $text,
                    )+
                }
            }
            /// the list of all the values of `Self`
            pub const VALUES: &'static [$keyword_enum] = &[
                $(
                    $keyword_enum::$name,
                )+
            ];
        }

        impl fmt::Display for $keyword_enum {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.pad(self.text())
            }
        }

        impl FromStr for $keyword_enum {
            type Err = ParseKeywordError;
            fn from_str(text: &str) -> Result<Self, ParseKeywordError> {
                match text {
                    $(
                        $text => Ok($keyword_enum::$name),
                    )+
                    _ => Err(ParseKeywordError),
                }
            }
        }
    };
}

keywords! {
    /// a keyword
    pub enum Keyword {
        Align = "align",
        Bool = "bool",
        Const = "const",
        DataPtr = "data_ptr",
        DontInline = "dont_inline",
        EntryPoint = "entry_point",
        F16 = "f16",
        F32 = "f32",
        F64 = "f64",
        False = "false",
        Fixed = "fixed",
        Fn = "fn",
        I16 = "i16",
        I32 = "i32",
        I64 = "i64",
        I8 = "i8",
        Inline = "inline",
        Kind = "kind",
        Module = "module",
        None = "none",
        Normal = "normal",
        Null = "null",
        Pure = "pure",
        RF32 = "rf32",
        RI32 = "ri32",
        Size = "size",
        Struct = "struct",
        True = "true",
        Undef = "undef",
        Variable = "variable",
        VScale = "vscale",
        X = "x",
    }
}

keywords! {
    /// an integer suffix
    pub enum IntegerSuffix {
        I8 = "i8",
        I16 = "i16",
        I32 = "i32",
        RI32 = "ri32",
        I64 = "i64",
    }
}

macro_rules! punctuation {
    (
        $(#[doc = $enum_doc:literal])*
        $vis:vis enum $enum:ident {
            $(#[doc = $doc1:expr] $name1:ident = $text1:literal,)*
            $name2:ident = $text2:literal,
            $($(#[doc = $doc3:expr])* $name3:ident = $text3:literal,)*
        }
    ) => {
        punctuation! {
            $(#[doc = $enum_doc])*
            $vis enum $enum {
                $(#[doc = $doc1] $name1 = $text1,)*
                #[doc = concat!("The punctuation \"", $text2, "\"")]
                $name2 = $text2,
                $($(#[doc = $doc3])* $name3 = $text3,)*
            }
        }
    };
    (
        $(#[doc = $enum_doc:literal])*
        $vis:vis enum $enum:ident {
            $(#[doc = $doc:expr] $name:ident = $text:literal,)+
        }
    ) => {
        $(#[doc = $enum_doc])*
        #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
        $vis enum $enum {
            $(
                #[doc = $doc]
                $name,
            )+
        }

        impl $enum {
            /// Get the textual form of `self`
            pub fn text(self) -> &'static str {
                match self {
                    $(
                        $enum::$name => $text,
                    )+
                }
            }
            /// the list of all the values of `Self`
            pub const VALUES: &'static [$enum] = &[
                $(
                    $enum::$name,
                )+
            ];
        }

        impl fmt::Display for $enum {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.pad(self.text())
            }
        }

        impl FromStr for $enum {
            type Err = ParseKeywordError;
            fn from_str(text: &str) -> Result<Self, ParseKeywordError> {
                match text {
                    $(
                        $text => Ok($enum::$name),
                    )+
                    _ => Err(ParseKeywordError),
                }
            }
        }
    };
}

punctuation! {
    /// a punctuation character or character sequence
    pub enum Punctuation {
        ExMark = "!",
        Dollar = "$",
        Percent = "%",
        Ampersand = "&",
        LParen = "(",
        RParen = ")",
        Asterisk = "*",
        Plus = "+",
        Comma = ",",
        Minus = "-",
        Period = ".",
        Slash = "/",
        Colon = ":",
        Semicolon = ";",
        LessThan = "<",
        Equal = "=",
        GreaterThan = ">",
        QMark = "?",
        At = "@",
        LSquareBracket = "[",
        RSquareBracket = "]",
        Caret = "^",
        Underscore = "_",
        LCurlyBrace = "{",
        VBar = "|",
        RCurlyBrace = "}",
        Tilde = "~",
        Arrow = "->",
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
enum IdentifierOrKeyword<'t> {
    Identifier(&'t str),
    Keyword(Keyword),
}

impl<'t> From<&'t str> for IdentifierOrKeyword<'t> {
    fn from(text: &'t str) -> IdentifierOrKeyword<'t> {
        match text.parse() {
            Ok(keyword) => IdentifierOrKeyword::Keyword(keyword),
            Err(_) => IdentifierOrKeyword::Identifier(text),
        }
    }
}

/// a string literal token
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct StringToken<'t> {
    /// the source text for the string literal excluding the enclosing quotes
    pub source_text: &'t str,
}

#[derive(Copy, Clone, Debug)]
struct ShortEscapeSequence {
    value: char,
}

macro_rules! short_escape_sequences {
    ($($source_char:literal => $value:literal,)+) => {
        impl ShortEscapeSequence {
            fn from_source(source_char: char) -> Option<ShortEscapeSequence> {
                match source_char {
                    $(
                        $source_char => Some(ShortEscapeSequence { value: $value }),
                    )+
                    _ => None,
                }
            }
        }
    };
}

short_escape_sequences! {
    '0' => '\0',
    'n' => '\n',
    'r' => '\r',
    't' => '\t',
    '\'' => '\'',
    '\"' => '\"',
    '\\' => '\\',
}

impl StringToken<'_> {
    /// the quote mark used to delimit string literals
    pub const QUOTE: char = '\"';
    fn parse_escape_sequence(location: &mut TextLocation) -> Result<char, &'static str> {
        if let Some(ShortEscapeSequence { value, .. }) =
            location.peek().and_then(ShortEscapeSequence::from_source)
        {
            location.next();
            return Ok(value);
        }
        match location.next() {
            None => return Err("truncated escape sequence"),
            Some('u') => {}
            _ => return Err("invalid escape sequence"),
        }
        match location.next() {
            None => return Err("truncated escape sequence"),
            Some('{') => {}
            _ => {
                return Err(
                    "invalid escape sequence; unicode escapes must be of the form \\u{1234}",
                )
            }
        }
        let digits_start_location = *location;
        while location.peek().map(|ch| ch.is_ascii_hexdigit()) == Some(true) {
            location.next();
        }
        let digits = TextSpan::new(digits_start_location, *location).text();
        if digits.is_empty() {
            return Err("invalid unicode escape sequence -- no hexadecimal digits");
        }
        match location.next() {
            None => Err("truncated escape sequence"),
            Some('}') => {
                let value = u32::from_str_radix(digits, 0x10)
                    .map_err(|_| "unicode escape value too big")?;
                if value > core::char::MAX as u32 {
                    return Err("unicode escape value too big");
                }
                core::char::from_u32(value).ok_or("invalid unicode escape value")
            }
            _ => Err("invalid escape sequence; unicode escapes must be of the form \\u{1234}"),
        }
    }
    fn parse_char(location: &mut TextLocation) -> Result<char, &'static str> {
        match location.next().ok_or("missing character")? {
            '\\' => Self::parse_escape_sequence(location),
            '\n' | '\r' => {
                Err(r#"line-ending not allowed in string, use "\n" and/or "\r" instead"#)
            }
            '\0' => Err(r#"null byte not allowed in string, use "\0" instead"#),
            ch => Ok(ch),
        }
    }
    /// get the decoded value of `self`
    ///
    /// # Panics
    ///
    /// Panics if `self.source_code` is not valid.
    pub fn value(self) -> String {
        let mut value = String::with_capacity(self.source_text.len());
        let source_code = FromTextSourceCode::new("", self.source_text);
        let mut location = TextLocation::new(0, &source_code);
        while location.peek().is_some() {
            value.push(
                Self::parse_char(&mut location).expect("StringToken should have valid source_text"),
            );
        }
        value
    }
}

/// an integer token
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct IntegerToken {
    /// the value of the integer -- not checked to be in-bounds for the suffix used
    pub value: u64,
    /// the suffix used for the integer token
    pub suffix: Option<IntegerSuffix>,
}

/// the kind of a token
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum TokenKind<'t> {
    /// a keyword
    Keyword(Keyword),
    /// an identifier
    Identifier(&'t str),
    /// the end of the source code
    EndOfFile,
    /// a potentially-suffixed integer
    Integer(IntegerToken),
    /// a string literal
    String(StringToken<'t>),
    /// a punctuation character or character sequence
    Punct(Punctuation),
}

impl<'t> TokenKind<'t> {
    /// return `Some` if `self` is a keyword
    pub fn keyword(self) -> Option<Keyword> {
        if let TokenKind::Keyword(retval) = self {
            Some(retval)
        } else {
            None
        }
    }
    /// return `Some` if `self` is an identifier
    pub fn identifier(self) -> Option<&'t str> {
        if let TokenKind::Identifier(retval) = self {
            Some(retval)
        } else {
            None
        }
    }
    /// return `Some` if `self` is an identifier or keyword, returning the textual form
    pub fn raw_identifier(self) -> Option<&'t str> {
        match self {
            Self::Identifier(v) => Some(v),
            Self::Keyword(v) => Some(v.text()),
            _ => None,
        }
    }
    /// return `true` if self is `EndOfFile`
    pub fn is_end_of_file(self) -> bool {
        if let TokenKind::EndOfFile = self {
            true
        } else {
            false
        }
    }
    /// return `Some` if `self` is an integer token
    pub fn integer(self) -> Option<IntegerToken> {
        if let TokenKind::Integer(retval) = self {
            Some(retval)
        } else {
            None
        }
    }
    /// return `Some` if `self` is a string literal
    pub fn string(self) -> Option<StringToken<'t>> {
        if let TokenKind::String(retval) = self {
            Some(retval)
        } else {
            None
        }
    }
    /// return `Some` if `self` is a `Punctuation`
    pub fn punct(self) -> Option<Punctuation> {
        if let TokenKind::Punct(retval) = self {
            Some(retval)
        } else {
            None
        }
    }
}

impl<'t> From<IdentifierOrKeyword<'t>> for TokenKind<'t> {
    fn from(value: IdentifierOrKeyword<'t>) -> TokenKind<'t> {
        match value {
            IdentifierOrKeyword::Identifier(identifier) => TokenKind::Identifier(identifier),
            IdentifierOrKeyword::Keyword(keyword) => TokenKind::Keyword(keyword),
        }
    }
}

/// a token, used for `FromText`
#[derive(Copy, Clone, Debug)]
pub struct Token<'t> {
    /// the text span
    pub span: TextSpan<'t>,
    /// the `TokenKind`
    pub kind: TokenKind<'t>,
}

/// the character used to start comments
pub const COMMENT_START_CHAR: char = '#';

/// the uninhabited type used for functions that always return `Err` or `Ok`
///
/// Use `Void::into` to convert to `!`, allowing Rust to coerce to any type
#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Debug)]
pub enum Void {}

impl fmt::Display for Void {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        match *self {}
    }
}

impl Void {
    /// convert to `!`, allowing Rust to coerce to any type
    pub fn into(self) -> ! {
        match self {}
    }
}

/// a scope id, for tracking where names are valid to use -- used for `FromText`
///
/// Use `FromTextSymbolsStateBase` to get the parent scope id, check if scope is visible, etc.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(transparent)]
pub struct FromTextScopeId {
    index: usize,
}

impl FromTextScopeId {
    /// the root scope, always visible. Doesn't have a parent scope.
    pub const ROOT: Self = Self { index: 0 };
}

/// the combination of a value and a scope id. Used as the value a name maps to.
pub struct FromTextSymbol<'g, T: Id<'g>> {
    /// the value that `self` represents.
    pub value: IdRef<'g, T>,
    /// the scope in which `self` is visible
    pub scope: FromTextScopeId,
}

impl<'g, T: Id<'g>> Clone for FromTextSymbol<'g, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'g, T: Id<'g>> Copy for FromTextSymbol<'g, T> {}

/// extension trait for `FromTextState`
pub trait FromTextSymbolsStateBase<'g, 't>: BorrowMut<FromTextState<'g, 't>> {
    /// get the parent scope id of `scope`
    fn get_parent_scope(&self, scope: FromTextScopeId) -> Option<FromTextScopeId> {
        if scope == FromTextScopeId::ROOT {
            None
        } else {
            Some(self.borrow().parent_scopes[scope.index])
        }
    }
    /// allocate a new scope id
    fn allocate_scope(&mut self, parent_scope: FromTextScopeId) -> FromTextScopeId {
        let parent_scopes = &mut self.borrow_mut().parent_scopes;
        let index = parent_scopes.len();
        debug_assert_ne!(index, FromTextScopeId::ROOT.index, "invalid state");
        parent_scopes.push(parent_scope);
        FromTextScopeId { index }
    }
    /// return true if `search_for_scope` is visible.
    /// A scope is visible if it is either `self.scope_stack_top` or
    /// a transitive parent of `self.scope_stack_top`.
    fn is_scope_visible(&self, search_for_scope: FromTextScopeId) -> bool {
        let mut scope = self.borrow().scope_stack_top;
        loop {
            if scope == search_for_scope {
                break true;
            }
            if let Some(parent_scope) = self.get_parent_scope(scope) {
                scope = parent_scope;
            } else {
                break false;
            }
        }
    }
    /// create a new scope with the parent set to `self.scope_stack_top`,
    /// then set `self.scope_stack_top` to the new scope id.
    fn push_new_nested_scope(&mut self) -> FromTextScopeId {
        let this = self.borrow_mut();
        let scope = this.allocate_scope(this.scope_stack_top);
        this.scope_stack_top = scope;
        scope
    }
}

#[doc(hidden)]
pub struct Private {
    _private: (),
}

impl Private {
    const fn new() -> Self {
        Self { _private: () }
    }
}

impl<'g, 't> FromTextSymbolsStateBase<'g, 't> for FromTextState<'g, 't> {}

/// extension trait for `FromTextState`
pub trait FromTextSymbolsState<'g, 't, T: Id<'g>>: FromTextSymbolsStateBase<'g, 't> {
    #[doc(hidden)]
    fn get_symbol_table(&self, _: Private) -> &HashMap<NamedId<'g>, FromTextSymbol<'g, T>>;
    #[doc(hidden)]
    fn get_symbol_table_mut(
        &mut self,
        _: Private,
    ) -> &mut HashMap<NamedId<'g>, FromTextSymbol<'g, T>>;
    /// get the `FromTextSymbol` corresponding to `name` in the symbol table for type `T`
    fn get_symbol(&self, name: NamedId<'g>) -> Option<FromTextSymbol<'g, T>> {
        self.get_symbol_table(Private::new()).get(&name).copied()
    }
    /// insert `name` and `symbol` in the symbol table for type `T`.
    /// returns `Err` without doing anything else if `name` was already in
    /// the symbol table for type `T`.
    fn insert_symbol(
        &mut self,
        name: NamedId<'g>,
        symbol: FromTextSymbol<'g, T>,
    ) -> Result<(), ()> {
        if let Entry::Vacant(entry) = self.get_symbol_table_mut(Private::new()).entry(name) {
            entry.insert(symbol);
            Ok(())
        } else {
            Err(())
        }
    }
}

/// state struct for `FromText`
pub struct FromTextState<'g, 't> {
    global_state: &'g GlobalState<'g>,
    /// the current `TextLocation`
    pub location: TextLocation<'t>,
    cached_token: Option<Token<'t>>,
    values: HashMap<NamedId<'g>, FromTextSymbol<'g, Value<'g>>>,
    blocks: HashMap<NamedId<'g>, FromTextSymbol<'g, BlockData<'g>>>,
    loops: HashMap<NamedId<'g>, FromTextSymbol<'g, LoopData<'g>>>,
    functions: HashMap<NamedId<'g>, FromTextSymbol<'g, FunctionData<'g>>>,
    pub(crate) structs: HashMap<u64, StructType<'g>>,
    parent_scopes: Vec<FromTextScopeId>,
    /// the scope id that defines what is currently visible.
    /// A scope is visible if it is either `self.scope_stack_top` or
    /// a transitive parent of `self.scope_stack_top`.
    pub scope_stack_top: FromTextScopeId,
}

impl<'g, 't> FromTextSymbolsState<'g, 't, Value<'g>> for FromTextState<'g, 't> {
    fn get_symbol_table(&self, _: Private) -> &HashMap<NamedId<'g>, FromTextSymbol<'g, Value<'g>>> {
        &self.values
    }
    fn get_symbol_table_mut(
        &mut self,
        _: Private,
    ) -> &mut HashMap<NamedId<'g>, FromTextSymbol<'g, Value<'g>>> {
        &mut self.values
    }
}

impl<'g, 't> FromTextSymbolsState<'g, 't, BlockData<'g>> for FromTextState<'g, 't> {
    fn get_symbol_table(
        &self,
        _: Private,
    ) -> &HashMap<NamedId<'g>, FromTextSymbol<'g, BlockData<'g>>> {
        &self.blocks
    }
    fn get_symbol_table_mut(
        &mut self,
        _: Private,
    ) -> &mut HashMap<NamedId<'g>, FromTextSymbol<'g, BlockData<'g>>> {
        &mut self.blocks
    }
}

impl<'g, 't> FromTextSymbolsState<'g, 't, LoopData<'g>> for FromTextState<'g, 't> {
    fn get_symbol_table(
        &self,
        _: Private,
    ) -> &HashMap<NamedId<'g>, FromTextSymbol<'g, LoopData<'g>>> {
        &self.loops
    }
    fn get_symbol_table_mut(
        &mut self,
        _: Private,
    ) -> &mut HashMap<NamedId<'g>, FromTextSymbol<'g, LoopData<'g>>> {
        &mut self.loops
    }
}

impl<'g, 't> FromTextSymbolsState<'g, 't, FunctionData<'g>> for FromTextState<'g, 't> {
    fn get_symbol_table(
        &self,
        _: Private,
    ) -> &HashMap<NamedId<'g>, FromTextSymbol<'g, FunctionData<'g>>> {
        &self.functions
    }
    fn get_symbol_table_mut(
        &mut self,
        _: Private,
    ) -> &mut HashMap<NamedId<'g>, FromTextSymbol<'g, FunctionData<'g>>> {
        &mut self.functions
    }
}

impl<'g, 't> FromTextState<'g, 't> {
    fn new(source_code: &'t FromTextSourceCode<'t>, global_state: &'g GlobalState<'g>) -> Self {
        Self {
            global_state,
            location: TextLocation::new(0, source_code),
            cached_token: None,
            values: HashMap::new(),
            blocks: HashMap::new(),
            loops: HashMap::new(),
            functions: HashMap::new(),
            structs: HashMap::new(),
            parent_scopes: vec![FromTextScopeId::ROOT],
            scope_stack_top: FromTextScopeId::ROOT,
        }
    }
    /// get the `GlobalState` reference
    pub fn global_state(&self) -> &'g GlobalState<'g> {
        self.global_state
    }
    /// create an error at the specified location with the specified message
    pub fn error_at<L: Into<FromTextErrorLocation>>(
        &mut self,
        location: L,
        message: impl ToString,
    ) -> Result<Void, FromTextError> {
        Err(FromTextError {
            location: location.into(),
            message: message.to_string(),
        })
    }
    fn peek_char(&self) -> Option<char> {
        self.location.peek()
    }
    fn next_char(&mut self) -> Option<char> {
        self.location.next()
    }
    /// create an error at the location of the next token with the specified message
    pub fn error_at_peek_token(&mut self, message: impl ToString) -> Result<Void, FromTextError> {
        let span = self.peek_token()?.span;
        self.error_at(span, message.to_string())
    }
    fn error_at_peek_char(&mut self, message: impl ToString) -> Result<Void, FromTextError> {
        self.error_at(self.location, message.to_string())
    }
    fn parse_comment(&mut self) -> Result<(), FromTextError> {
        if self.peek_char() != Some(COMMENT_START_CHAR) {
            self.error_at_peek_char("missing comment")?;
        }
        loop {
            match self.next_char() {
                None | Some('\n') => break,
                _ => {}
            }
        }
        Ok(())
    }
    fn skip_whitespace(&mut self) -> Result<(), FromTextError> {
        loop {
            match self.peek_char() {
                Some(COMMENT_START_CHAR) => self.parse_comment()?,
                Some(ch) => {
                    if !ch.is_ascii_whitespace() {
                        break;
                    }
                }
                None => break,
            }
            self.next_char();
        }
        Ok(())
    }
    fn parse_raw_identifier(&mut self) -> Result<&'t str, FromTextError> {
        let start_location = self.location;
        if self.peek_char().map(char::is_identifier_start) != Some(true) {
            self.error_at_peek_char("missing identifier")?;
        }
        while self.peek_char().map(char::is_identifier_continue) == Some(true) {
            self.next_char();
        }
        Ok(TextSpan::new(start_location, self.location).text())
    }
    fn parse_identifier_or_keyword(&mut self) -> Result<IdentifierOrKeyword<'t>, FromTextError> {
        self.parse_raw_identifier().map(Into::into)
    }
    fn parse_optional_integer_suffix(&mut self) -> Result<Option<IntegerSuffix>, FromTextError> {
        let start_location = self.location;
        if self.peek_char().map(char::is_identifier_start) != Some(true) {
            return Ok(None);
        }
        while self.peek_char().map(char::is_identifier_continue) == Some(true) {
            self.next_char();
        }
        let span = TextSpan::new(start_location, self.location);
        match span.text().parse::<IntegerSuffix>() {
            Ok(retval) => Ok(Some(retval)),
            Err(_) => self.error_at(span, "invalid integer suffix")?.into(),
        }
    }
    fn parse_integer(&mut self) -> Result<IntegerToken, FromTextError> {
        if self.peek_char().map(|ch| ch.is_ascii_digit()) != Some(true) {
            self.error_at_peek_char("expected number")?;
        }
        let mut digits_start_location = self.location;
        let radix;
        if self.peek_char() == Some('0') {
            self.next_char();
            match self.peek_char() {
                Some('x') | Some('X') => {
                    self.next_char();
                    digits_start_location = self.location;
                    radix = 16;
                }
                Some('o') | Some('O') => {
                    self.next_char();
                    digits_start_location = self.location;
                    radix = 8;
                }
                Some('b') | Some('B') => {
                    self.next_char();
                    digits_start_location = self.location;
                    radix = 2;
                }
                Some(ch) if ch.is_ascii_digit() => self
                    .error_at_peek_char("octal numbers must start with 0o or 0O")?
                    .into(),
                _ => {
                    return Ok(IntegerToken {
                        value: 0,
                        suffix: self.parse_optional_integer_suffix()?,
                    })
                }
            }
        } else {
            radix = 10;
        }
        while self
            .location
            .peek()
            .and_then(|ch| ch.to_digit(radix))
            .is_some()
        {
            self.next_char();
        }
        let digits = TextSpan::new(digits_start_location, self.location).text();
        let suffix = self.parse_optional_integer_suffix()?;
        match u64::from_str_radix(digits, radix) {
            Ok(value) => Ok(IntegerToken { value, suffix }),
            _ => self
                .error_at(digits_start_location, "number too big")?
                .into(),
        }
    }
    fn parse_string(&mut self) -> Result<StringToken<'t>, FromTextError> {
        if self.peek_char() != Some(StringToken::QUOTE) {
            self.error_at_peek_char("missing string")?;
        }
        let quote_location = self.location;
        self.next_char();
        let string_body_start_location = self.location;
        loop {
            match self.peek_char() {
                None => self.error_at(quote_location, "unterminated string")?.into(),
                Some(StringToken::QUOTE) => {
                    let string_body_end_location = self.location;
                    self.next_char();
                    return Ok(StringToken {
                        source_text: TextSpan::new(
                            string_body_start_location,
                            string_body_end_location,
                        )
                        .text(),
                    });
                }
                _ => match StringToken::parse_char(&mut self.location) {
                    Ok(_) => {}
                    Err(message) => self.error_at_peek_char(message)?.into(),
                },
            }
        }
    }
    fn parse_punct(&mut self) -> Result<Punctuation, FromTextError> {
        if self.peek_char().is_none() {
            self.error_at_peek_char("missing punctuation")?;
        }
        let start_location = self.location;
        let mut matched = None;
        while self.next_char().is_some() {
            let peek_text = TextSpan::new(start_location, self.location).text();
            let mut is_prefix = false;
            for &punct in Punctuation::VALUES {
                let punct_text = punct.text();
                if peek_text == punct_text {
                    matched = Some((punct, self.location));
                } else if punct_text.starts_with(peek_text) {
                    is_prefix = true;
                }
            }
            if !is_prefix {
                break;
            }
        }
        if let Some((retval, end_location)) = matched {
            self.location = end_location;
            Ok(retval)
        } else {
            self.location = start_location;
            self.error_at_peek_char("invalid punctuation")?.into()
        }
    }
    fn parse_token_impl(&mut self) -> Result<Token<'t>, FromTextError> {
        self.skip_whitespace()?;
        let start_location = self.location;
        match self.peek_char() {
            None => Ok(Token {
                kind: TokenKind::EndOfFile,
                span: TextSpan::new(start_location, self.location),
            }),
            Some(StringToken::QUOTE) => Ok(Token {
                kind: TokenKind::String(self.parse_string()?),
                span: TextSpan::new(start_location, self.location),
            }),
            Some(ch) if ch.is_identifier_start() => Ok(Token {
                kind: self.parse_identifier_or_keyword()?.into(),
                span: TextSpan::new(start_location, self.location),
            }),
            Some(ch) if ch.is_ascii_digit() => Ok(Token {
                kind: TokenKind::Integer(self.parse_integer()?),
                span: TextSpan::new(start_location, self.location),
            }),
            _ => Ok(Token {
                kind: TokenKind::Punct(self.parse_punct()?),
                span: TextSpan::new(start_location, self.location),
            }),
        }
    }
    /// return the next token, but resetting `self.location` to the beginning
    /// of the next token so that it is returned again at the next
    /// `peek_token` or `parse_token` call.
    /// However, this does advance `self.location` past any intervening comments or whitespace.
    pub fn peek_token(&mut self) -> Result<Token<'t>, FromTextError> {
        if let Some(cached_token) = self.cached_token {
            if cached_token.span.start() == self.location {
                return Ok(cached_token);
            }
        }
        let token = self.parse_token_impl()?;
        self.location = token.span.start();
        self.cached_token = Some(token);
        Ok(token)
    }
    /// parse the next token, advancing `self.location` to right after it.
    pub fn parse_token(&mut self) -> Result<Token<'t>, FromTextError> {
        if let Some(cached_token) = self.cached_token.take() {
            if cached_token.span.start() == self.location {
                self.location = cached_token.span.end();
                return Ok(cached_token);
            }
        }
        self.parse_token_impl()
    }
    /// parse the next token, erroring if it is not the passed-in `punct`
    pub fn parse_punct_token_or_error(
        &mut self,
        punct: Punctuation,
        error_msg: impl ToString,
    ) -> Result<Token<'t>, FromTextError> {
        let token = self.parse_token()?;
        if token.kind.punct() != Some(punct) {
            self.error_at(token.span, error_msg)?;
        }
        Ok(token)
    }
    /// parse the next token, erroring if it is not the passed-in `keyword`
    pub fn parse_keyword_token_or_error(
        &mut self,
        keyword: Keyword,
        error_msg: impl ToString,
    ) -> Result<Token<'t>, FromTextError> {
        let token = self.parse_token()?;
        if token.kind.keyword() != Some(keyword) {
            self.error_at(token.span, error_msg)?;
        }
        Ok(token)
    }
    /// parse `open_paren` then call `body` then parse `close_paren`.
    /// Useful for parsing source that is grouped using delimiter
    /// punctuation, such as `"(i8)"`.
    pub fn parse_parenthesized<T, F: FnOnce(&mut Self) -> Result<T, FromTextError>>(
        &mut self,
        open_paren: Punctuation,
        missing_open_paren_error_msg: impl ToString,
        close_paren: Punctuation,
        missing_close_paren_error_msg: impl ToString,
        body: F,
    ) -> Result<T, FromTextError> {
        self.parse_punct_token_or_error(open_paren, missing_open_paren_error_msg)?;
        let retval = body(self)?;
        self.parse_punct_token_or_error(close_paren, missing_close_paren_error_msg)?;
        Ok(retval)
    }
}

/// parse text
pub trait FromText<'g>: FromToTextListForm {
    /// the type produced by parsing text successfully
    type Parsed: Sized;
    /// top-level parse function -- should not be called from `from_text` implementations
    fn parse(
        file_name: impl Borrow<str>,
        text: impl Borrow<str>,
        global_state: &'g GlobalState<'g>,
    ) -> Result<Self::Parsed, FromTextError> {
        let file_name = file_name.borrow();
        let text = text.borrow();
        let source_code = FromTextSourceCode::new(file_name, text);
        let mut state = FromTextState::new(&source_code, global_state);
        let retval = Self::from_text(&mut state)?;
        if !state.peek_token()?.kind.is_end_of_file() {
            state.error_at_peek_token("extra tokens at end")?;
        }
        Ok(retval)
    }
    /// do the actual parsing work
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self::Parsed, FromTextError>;
}

/// a name plus the integer suffix
#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct NamedId<'g> {
    /// the name
    pub name: Interned<'g, str>,
    /// the integer suffix. is `0` when the name doesn't have a suffix.
    pub name_suffix: u64,
}

impl<'g> NamedId<'g> {
    /// check if the name must be in quoted form (`"non-id-name"234` instead of `name`).
    /// This is true if the name is not a valid
    /// raw identifier (`TokenKind::raw_identifier`) or `self.name_suffix` is non-zero.
    pub fn needs_quoted_form(self) -> bool {
        let NamedId { name, name_suffix } = self;
        if name_suffix != 0 {
            true
        } else {
            let mut chars = name.chars();
            if let Some(first) = chars.next() {
                if !first.is_identifier_start() {
                    true
                } else {
                    !chars.all(|ch| ch.is_identifier_continue())
                }
            } else {
                true
            }
        }
    }
}

impl FromToTextListForm for NamedId<'_> {}

impl<'g> FromText<'g> for NamedId<'g> {
    type Parsed = Self;
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError> {
        match state.peek_token()?.kind {
            TokenKind::Identifier(name) => {
                state.parse_token()?;
                Ok(Self {
                    name: name.intern(state.global_state()),
                    name_suffix: 0,
                })
            }
            TokenKind::Keyword(name) => {
                state.parse_token()?;
                Ok(Self {
                    name: name.text().intern(state.global_state()),
                    name_suffix: 0,
                })
            }
            TokenKind::String(name) => {
                state.parse_token()?;
                if let Some(IntegerToken { value, suffix }) = state.peek_token()?.kind.integer() {
                    if suffix.is_some() {
                        state.error_at_peek_token(r#"name suffix must be unsuffixed integer ("my_name"123 and not "my_name"123i8)"#)?;
                    }
                    state.parse_token()?;
                    Ok(Self {
                        name: name.value().intern(state.global_state()),
                        name_suffix: value,
                    })
                } else {
                    state.error_at_peek_token("missing name suffix")?.into()
                }
            }
            _ => state
                .error_at_peek_token("missing name -- must be identifier or string")?
                .into(),
        }
    }
}

impl_display_as_to_text!(<'g> NamedId<'g>);

impl<'g> ToText<'g> for NamedId<'g> {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        if self.needs_quoted_form() {
            self.name.to_text(state)?;
            write!(state, "{}", self.name_suffix)
        } else {
            write!(state, "{}", self.name)
        }
    }
}

impl FromToTextListForm for str {}

impl<'g> ToText<'g> for str {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        write!(state, "\"{}\"", self.escape_default())
    }
}

trait NameMapGetName<'g>: Id<'g> {
    fn name(&self) -> Interned<'g, str>;
}

impl<'g> NameMapGetName<'g> for Value<'g> {
    fn name(&self) -> Interned<'g, str> {
        self.name
    }
}

impl<'g> NameMapGetName<'g> for BlockData<'g> {
    fn name(&self) -> Interned<'g, str> {
        self.name
    }
}

impl<'g> NameMapGetName<'g> for LoopData<'g> {
    fn name(&self) -> Interned<'g, str> {
        self.name
    }
}

impl<'g> NameMapGetName<'g> for FunctionData<'g> {
    fn name(&self) -> Interned<'g, str> {
        self.name
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) enum NewOrOld<T> {
    New(T),
    Old(T),
}

struct NameMap<'g, T: NameMapGetName<'g>> {
    named_ids: HashMap<IdRef<'g, T>, NamedId<'g>>,
    name_suffixes: HashMap<Interned<'g, str>, u64>,
}

impl<'g, T: NameMapGetName<'g>> NameMap<'g, T> {
    fn new() -> Self {
        Self {
            named_ids: HashMap::new(),
            name_suffixes: HashMap::new(),
        }
    }
    fn get(&mut self, value: IdRef<'g, T>) -> NewOrOld<NamedId<'g>> {
        match self.named_ids.entry(value) {
            Entry::Occupied(entry) => NewOrOld::Old(*entry.get()),
            Entry::Vacant(entry) => {
                let name = value.name();
                let next_name_suffix = self.name_suffixes.entry(name).or_insert(0);
                let name_suffix = *next_name_suffix;
                *next_name_suffix += 1;
                NewOrOld::New(*entry.insert(NamedId { name, name_suffix }))
            }
        }
    }
}

/// state struct for `ToText`
pub struct ToTextState<'g, 'w> {
    indent: usize,
    at_start_of_line: bool,
    base_writer: &'w mut dyn FnMut(&str) -> fmt::Result,
    values: NameMap<'g, Value<'g>>,
    blocks: NameMap<'g, BlockData<'g>>,
    loops: NameMap<'g, LoopData<'g>>,
    functions: NameMap<'g, FunctionData<'g>>,
    struct_type_ids: HashMap<StructType<'g>, usize>,
    is_fragment: bool,
}

impl<'g, 'w> ToTextState<'g, 'w> {
    /// the number of spaces used as an indentation unit.
    pub const INDENT_MULTIPLE: usize = 4;
    /// true if `ToText` is being used on only a piece of IR rather than the whole IR.
    /// Used to turn off assertions that names are defined first.
    pub fn is_fragment(&self) -> bool {
        self.is_fragment
    }
    pub(crate) fn check_name_definition<T>(&self, value: NewOrOld<T>, error_message: &str) -> T {
        match value {
            NewOrOld::New(v) => v,
            NewOrOld::Old(v) => {
                if !self.is_fragment() {
                    panic!("{}", error_message);
                } else {
                    v
                }
            }
        }
    }
    pub(crate) fn check_name_use<T>(&self, value: NewOrOld<T>, error_message: &str) -> T {
        match value {
            NewOrOld::Old(v) => v,
            NewOrOld::New(v) => {
                if !self.is_fragment() {
                    panic!("{}", error_message);
                } else {
                    v
                }
            }
        }
    }
    pub(crate) fn new(
        base_writer: &'w mut dyn FnMut(&str) -> fmt::Result,
        is_fragment: bool,
    ) -> Self {
        ToTextState {
            indent: 0,
            at_start_of_line: true,
            base_writer,
            values: NameMap::new(),
            blocks: NameMap::new(),
            loops: NameMap::new(),
            functions: NameMap::new(),
            struct_type_ids: HashMap::new(),
            is_fragment,
        }
    }
    pub(crate) fn get_value_named_id(
        &mut self,
        value: IdRef<'g, Value<'g>>,
    ) -> NewOrOld<NamedId<'g>> {
        self.values.get(value)
    }
    pub(crate) fn get_block_named_id(
        &mut self,
        value: IdRef<'g, BlockData<'g>>,
    ) -> NewOrOld<NamedId<'g>> {
        self.blocks.get(value)
    }
    pub(crate) fn get_loop_named_id(
        &mut self,
        value: IdRef<'g, LoopData<'g>>,
    ) -> NewOrOld<NamedId<'g>> {
        self.loops.get(value)
    }
    pub(crate) fn get_function_named_id(
        &mut self,
        value: IdRef<'g, FunctionData<'g>>,
    ) -> NewOrOld<NamedId<'g>> {
        self.functions.get(value)
    }
    pub(crate) fn get_struct_type_id(&mut self, struct_type: &StructType<'g>) -> NewOrOld<usize> {
        let next_id = self.struct_type_ids.len();
        if let Some(retval) = self.struct_type_ids.get(struct_type) {
            NewOrOld::Old(*retval)
        } else {
            self.struct_type_ids.insert(struct_type.clone(), next_id);
            NewOrOld::New(next_id)
        }
    }
    /// indent the text produced by the passed-in function by 1 unit (`Self::INDENT_MULTIPLE` spaces).
    ///
    /// # Panics
    ///
    /// Panics if the text output is not at the start of a line both before and
    /// after the passed-in function is called (only checked if the passed-in function succeeds).
    pub fn indent<R, E, F: FnOnce(&mut Self) -> Result<R, E>>(&mut self, f: F) -> Result<R, E> {
        assert!(
            self.at_start_of_line,
            "can't call indent() in the middle of a text line"
        );
        self.indent += 1;
        let retval = f(self)?;
        assert!(
            self.at_start_of_line,
            "can't return Ok to indent() in the middle of a text line"
        );
        self.indent -= 1;
        Ok(retval)
    }
    /// rebind `core::fmt::Write::write_fmt` to make it easily visible for use with the `write!` and `writeln!` macros
    #[inline]
    pub fn write_fmt(&mut self, args: fmt::Arguments) -> fmt::Result {
        fmt::Write::write_fmt(self, args)
    }
}

impl fmt::Write for ToTextState<'_, '_> {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        let mut first = true;
        for text in text.split('\n') {
            if !mem::replace(&mut first, false) {
                (self.base_writer)("\n")?;
                self.at_start_of_line = true;
            }
            if text.is_empty() {
                continue;
            }
            let do_indent = mem::replace(&mut self.at_start_of_line, false);
            if do_indent && self.indent != 0 {
                // 256 spaces arranged in a 16x16 grid
                const SPACES: &str = concat!(
                    "                ",
                    "                ",
                    "                ",
                    "                ",
                    //
                    "                ",
                    "                ",
                    "                ",
                    "                ",
                    //
                    "                ",
                    "                ",
                    "                ",
                    "                ",
                    //
                    "                ",
                    "                ",
                    "                ",
                    "                ",
                );

                // write in larger chunks to speed-up output

                let mut indent = self.indent * Self::INDENT_MULTIPLE;
                while indent >= SPACES.len() {
                    (self.base_writer)(SPACES)?;
                    indent -= SPACES.len();
                }
                (self.base_writer)(&SPACES[..indent])?;
            }
            (self.base_writer)(text)?;
        }
        Ok(())
    }
}

/// the textual form of a list
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ListForm {
    /// the opening punctuation for a list
    ///
    /// For the list `[a, b]`, the opening punctuation is the `[`.
    pub opening_punct: Punctuation,
    /// the error message for when the opening punctuation is missing
    pub opening_punct_missing_msg: &'static str,
    /// the closing punctuation for a list
    ///
    /// For the list `[a, b]`, the closing punctuation is the `]`.
    pub closing_punct: Punctuation,
    /// the error message for when the closing punctuation is missing
    pub closing_punct_missing_msg: &'static str,
    /// the punctuation that comes after each item in a list
    ///
    /// for the list `[a, b, c]`, the item terminator is the `,`.
    pub item_terminator: Punctuation,
    /// the error message for when an item terminator is missing
    pub item_terminator_missing_msg: &'static str,
    /// If the final item terminator is required when parsing
    ///
    /// If this is set, then lists like `[a, b, c]` are not allowed because there isn't a `,` after the `c`.
    pub final_item_terminator_required: bool,
    /// If the final item terminator is produced when converting to text
    ///
    /// If this is set, then lists are produced like `[a, b, c,]` instead of like `[a, b, c]`.
    pub final_item_terminator_displayed: bool,
    /// If the list, when converted to text, is spread over multiple lines instead of all on one line.
    ///
    /// If this is set, then lists are produced like:
    /// ```text
    /// {
    ///     a;
    ///     b;
    ///     c;
    /// }
    /// ```
    /// instead of:
    /// ```text
    /// {a; b; c;}
    /// ```
    pub multi_line: bool,
}

impl ListForm {
    /// use `[a, b, c]`
    pub const SQUARE_BRACKETS: ListForm = ListForm {
        opening_punct: Punctuation::LSquareBracket,
        opening_punct_missing_msg: "missing opening square bracket (`[`)",
        closing_punct: Punctuation::RSquareBracket,
        closing_punct_missing_msg: "missing closing square bracket (`]`)",
        item_terminator: Punctuation::Comma,
        item_terminator_missing_msg: "missing comma after item (`,`)",
        final_item_terminator_required: false,
        final_item_terminator_displayed: false,
        multi_line: false,
    };
    /// use:
    /// ```text
    /// {
    ///     a;
    ///     b;
    ///     c;
    /// }
    /// ```
    pub const STATEMENTS: ListForm = ListForm {
        opening_punct: Punctuation::LCurlyBrace,
        opening_punct_missing_msg: "missing opening curly brace (`{`)",
        closing_punct: Punctuation::RCurlyBrace,
        closing_punct_missing_msg: "missing closing curly brace (`}`)",
        item_terminator: Punctuation::Semicolon,
        item_terminator_missing_msg: "missing semicolon after item (`;`)",
        final_item_terminator_required: true,
        final_item_terminator_displayed: true,
        multi_line: true,
    };
    /// parse a list, calling `parse_item` to parse each item in the list
    pub fn parse_list<'g, 'a>(
        self,
        state: &mut FromTextState<'g, 'a>,
        parse_item: impl FnMut(&mut FromTextState<'g, 'a>) -> Result<(), FromTextError>,
    ) -> Result<(), FromTextError> {
        self.parse_list_with_extra_callbacks(state, |_| Ok(()), parse_item)
    }
    /// parse a list, calling `after_opening_punct` after parsing `self.opening_punct`,
    /// calling `parse_item` to parse each item in the list
    pub fn parse_list_with_extra_callbacks<'g, 'a, R>(
        self,
        state: &mut FromTextState<'g, 'a>,
        after_opening_punct: impl FnOnce(&mut FromTextState<'g, 'a>) -> Result<R, FromTextError>,
        mut parse_item: impl FnMut(&mut FromTextState<'g, 'a>) -> Result<(), FromTextError>,
    ) -> Result<R, FromTextError> {
        state.parse_parenthesized(
            self.opening_punct,
            self.opening_punct_missing_msg,
            self.closing_punct,
            self.closing_punct_missing_msg,
            |state| {
                let retval = after_opening_punct(state)?;
                loop {
                    if state.peek_token()?.kind.punct() == Some(self.closing_punct) {
                        break;
                    }
                    parse_item(state)?;
                    let peek_punct = state.peek_token()?.kind.punct();
                    if peek_punct == Some(self.closing_punct) {
                        if self.final_item_terminator_required {
                            state.error_at_peek_token(self.item_terminator_missing_msg)?;
                        }
                    } else if self.final_item_terminator_required
                        || peek_punct == Some(self.item_terminator)
                    {
                        state.parse_punct_token_or_error(
                            self.item_terminator,
                            self.item_terminator_missing_msg,
                        )?;
                        continue;
                    }
                    break;
                }
                Ok(retval)
            },
        )
    }
    /// parse a list, producing a `Vec`
    pub fn parse_vec<'g, 'a, Item>(
        self,
        state: &mut FromTextState<'g, 'a>,
        mut parse_item: impl FnMut(&mut FromTextState<'g, 'a>) -> Result<Item, FromTextError>,
    ) -> Result<Vec<Item>, FromTextError> {
        let mut retval = Vec::new();
        self.parse_list(state, |state| {
            retval.push(parse_item(state)?);
            Ok(())
        })?;
        Ok(retval)
    }
    /// parse a list, producing a fixed-length array `[T; N]`
    pub fn parse_array<'g, T: FromText<'g> + ?Sized, A: Array<Item = T::Parsed>>(
        self,
        state: &mut FromTextState<'g, '_>,
    ) -> Result<A, FromTextError> {
        let mut retval = ArrayVec::<A>::new();
        let list_location = state.peek_token()?.span;
        self.parse_list(state, |state| {
            if retval.is_full() {
                state.error_at_peek_token(if A::CAPACITY == 1 {
                    format!("list too long, expected only {} item", A::CAPACITY)
                } else {
                    format!("list too long, expected only {} items", A::CAPACITY)
                })?;
            }
            retval.push(T::from_text(state)?);
            Ok(())
        })?;
        match retval.into_inner() {
            Ok(retval) => Ok(retval),
            Err(retval) => state
                .error_at(
                    list_location,
                    if A::CAPACITY == 1 {
                        format!(
                            "list too short, expected {} item, got {}",
                            A::CAPACITY,
                            retval.len()
                        )
                    } else {
                        format!(
                            "list too short, expected {} items, got {}",
                            A::CAPACITY,
                            retval.len()
                        )
                    },
                )?
                .into(),
        }
    }
    /// write a list
    pub fn list_to_text<'g, Item: ToText<'g>>(
        self,
        state: &mut ToTextState<'g, '_>,
        items: impl IntoIterator<Item = Item>,
    ) -> fmt::Result {
        self.list_to_text_with_extra_callbacks(
            state,
            |_| Ok(()),
            |state, item| item.to_text(state),
            items,
        )
    }
    /// write a list, calling `after_opening_punct` after writing `self.opening_punct`,
    /// calling `item_to_text` to write each item in the list
    pub fn list_to_text_with_extra_callbacks<'g, 'a, Item>(
        self,
        state: &mut ToTextState<'g, 'a>,
        after_opening_punct: impl FnOnce(&mut ToTextState<'g, 'a>) -> fmt::Result,
        mut item_to_text: impl FnMut(&mut ToTextState<'g, 'a>, Item) -> fmt::Result,
        items: impl IntoIterator<Item = Item>,
    ) -> fmt::Result {
        let write_body = |state: &mut ToTextState<'g, 'a>| -> fmt::Result {
            after_opening_punct(state)?;
            let mut items = items.into_iter().peekable();
            while let Some(item) = items.next() {
                item_to_text(state, item)?;
                if self.final_item_terminator_displayed || items.peek().is_some() {
                    write!(state, "{}", self.item_terminator)?;
                }
                if self.multi_line {
                    writeln!(state)?;
                } else if items.peek().is_some() {
                    write!(state, " ")?;
                }
            }
            Ok(())
        };
        if self.multi_line {
            writeln!(state, "{}", self.opening_punct)?;
            state.indent(write_body)?;
        } else {
            write!(state, "{}", self.opening_punct)?;
            write_body(state)?;
        }
        write!(state, "{}", self.closing_punct)
    }
}

impl Default for ListForm {
    fn default() -> Self {
        ListForm::SQUARE_BRACKETS
    }
}

/// get the default `ListForm` that should be used when using `FromText` or `ToText` on lists of `Self`
pub trait FromToTextListForm {
    /// get the default `ListForm` that should be used when using `FromText` or `ToText` on lists of `Self`
    fn from_to_text_list_form() -> ListForm {
        ListForm::default()
    }
}

/// trait for converting IR to text
///
/// To convert to a `String` or print or write, use `ToText::display`
pub trait ToText<'g>: FromToTextListForm {
    /// produce a value that can be used with `core::fmt`.
    ///
    /// should not be used from `ToText` implementations, `ToText::to_text` should instead be called.
    fn display(&self) -> ToTextDisplay<'g, '_, Self> {
        ToTextDisplay::new(self, true)
    }
    /// convert `self` to text.
    ///
    /// `ToText` implementations should call `ToText::to_text` rather than `ToText::display`.
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result;
}

/// helper struct to allow a type implementing `ToText` to be used with `core::fmt`.
pub struct ToTextDisplay<'g, 'a, T: ToText<'g> + ?Sized> {
    value: &'a T,
    is_fragment: bool,
    _phantom: PhantomData<&'g ()>,
}

impl<'g, 'a, T: ToText<'g> + ?Sized> ToTextDisplay<'g, 'a, T> {
    pub(crate) fn new(value: &'a T, is_fragment: bool) -> Self {
        Self {
            value,
            is_fragment,
            _phantom: PhantomData,
        }
    }
}

impl<'g, T: ToText<'g> + ?Sized> fmt::Display for ToTextDisplay<'g, '_, T> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.value.to_text(&mut ToTextState::new(
            &mut |text: &str| formatter.write_str(text),
            self.is_fragment,
        ))
    }
}

impl<'g, T: ToText<'g> + ?Sized> fmt::Debug for ToTextDisplay<'g, '_, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl<T: FromToTextListForm + ?Sized> FromToTextListForm for &'_ T {
    fn from_to_text_list_form() -> ListForm {
        T::from_to_text_list_form()
    }
}

impl<'g, T: ToText<'g> + ?Sized> ToText<'g> for &'_ T {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        (**self).to_text(state)
    }
}

impl<'g, T: ToText<'g>> ToText<'g> for [T] {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        T::from_to_text_list_form().list_to_text(state, self)
    }
}

impl<'g, T: ToText<'g>> ToText<'g> for Vec<T> {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        (**self).to_text(state)
    }
}
impl<T: FromToTextListForm> FromToTextListForm for [T] {
    fn from_to_text_list_form() -> ListForm {
        T::from_to_text_list_form()
    }
}

impl<'g, T: FromText<'g>> FromText<'g> for [T] {
    type Parsed = Vec<T::Parsed>;
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Vec<T::Parsed>, FromTextError> {
        T::from_to_text_list_form().parse_vec(state, T::from_text)
    }
}

impl<T: FromToTextListForm> FromToTextListForm for Vec<T> {
    fn from_to_text_list_form() -> ListForm {
        T::from_to_text_list_form()
    }
}

impl<'g, T: FromText<'g>> FromText<'g> for Vec<T> {
    type Parsed = Vec<T::Parsed>;
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Vec<T::Parsed>, FromTextError> {
        <[T]>::from_text(state)
    }
}

macro_rules! impl_from_to_text_for_arrays {
    ($($n:literal,)*) => {
        $(
            impl<T: FromToTextListForm> FromToTextListForm for [T; $n] {
                fn from_to_text_list_form() -> ListForm {
                    T::from_to_text_list_form()
                }
            }

            impl<'g, T: ToText<'g>> ToText<'g> for [T; $n] {
                fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
                    (self as &[T]).to_text(state)
                }
            }

            impl<'g, T: FromText<'g>> FromText<'g> for [T; $n] {
                type Parsed = [T::Parsed; $n];
                fn from_text(
                    state: &mut FromTextState<'g, '_>,
                ) -> Result<[T::Parsed; $n], FromTextError> {
                    T::from_to_text_list_form().parse_array::<T, [T::Parsed; $n]>(state)
                }
            }
        )*
    };
}

impl_from_to_text_for_arrays!(
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31,
);
