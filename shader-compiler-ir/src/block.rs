// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::prelude::*;
use crate::text::FromTextError;
use crate::text::FromTextState;
use crate::text::FromTextSymbol;
use crate::text::FromTextSymbolsState;
use crate::text::FromTextSymbolsStateBase;
use crate::text::NamedId;
use crate::text::NewOrOld;
use crate::text::Punctuation;
use crate::text::ToTextState;
use crate::text::Token;
use crate::text::TokenKind;
use crate::Allocate;
use crate::InstructionKind;
use crate::OnceCell;
use std::fmt;
use std::ops::Deref;

/// break out of a block.
/// jumps to the first instruction after `self.block`.
#[derive(Debug)]
pub struct BreakBlock<'g> {
    /// the block to break out of
    pub block: BlockRef<'g>,
    /// the values the block will return
    pub block_results: Vec<ValueUse<'g>>,
}

impl<'g> ToText<'g> for BreakBlock<'g> {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        Self::KIND.to_text(state)?;
        write!(state, " ")?;
        let Self {
            block,
            block_results,
        } = self;
        block.to_text(state)?;
        block_results.to_text(state)
    }
}

impl<'g> FromText<'g> for BreakBlock<'g> {
    type Parsed = Self;
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError> {
        let kind_location = state.peek_token()?.span;
        if Self::KIND != InstructionKind::from_text(state)? {
            state.error_at(
                kind_location,
                format!("expected {} instruction", Self::KIND.text()),
            )?;
        }
        let block = BlockRef::from_text(state)?;
        let block_results = Vec::<ValueUse<'g>>::from_text(state)?;
        Ok(Self {
            block,
            block_results,
        })
    }
}

impl<'g> CodeIO<'g> for BreakBlock<'g> {
    fn results(&self) -> Inhabitable<&[ValueDefinition<'g>]> {
        Uninhabited
    }
    fn arguments(&self) -> &[ValueUse<'g>] {
        &self.block_results
    }
}

/// the header of a loop, holds the `ValueDefinition`s assigned at the beginning of each loop iteration
#[derive(Eq, PartialEq, Hash, Debug)]
pub struct LoopHeader<'g> {
    /// the `ValueDefinition`s assigned at the beginning of each loop iteration
    pub argument_definitions: Vec<ValueDefinition<'g>>,
}

impl<'g> CodeIO<'g> for LoopHeader<'g> {
    fn results(&self) -> Inhabitable<&[ValueDefinition<'g>]> {
        Inhabited(&self.argument_definitions)
    }
    fn arguments(&self) -> &[ValueUse<'g>] {
        &[]
    }
}

/// the struct storing the data for a `Block`
#[derive(Debug)]
pub struct BlockData<'g> {
    /// the name of the `Block` -- doesn't need to be unique
    pub name: Interned<'g, str>,
    /// the body of the `Block`
    pub body: OnceCell<Vec<Instruction<'g>>>,
    /// The `ValueDefinition`s assigned to by `BreakBlock` when the block finishes executing.
    /// Is `Uninhabited` if there is no `BreakBlock` targeting `self`.
    pub result_definitions: Inhabitable<Vec<ValueDefinition<'g>>>,
}

impl<'g> BlockData<'g> {
    /// Sets the body of `self` to the passed-in value.
    ///
    /// # Panics
    ///
    /// Panics if the body was already set.
    pub fn set_body(&self, body: Vec<Instruction<'g>>) {
        self.body.set(body).ok().expect("block body already set");
    }
}

impl<'g> Id<'g> for BlockData<'g> {}

impl<'g> CodeIO<'g> for BlockData<'g> {
    fn results(&self) -> Inhabitable<&[ValueDefinition<'g>]> {
        self.result_definitions.as_deref()
    }
    fn arguments(&self) -> &[ValueUse<'g>] {
        &[]
    }
}

/// a reference to a `Block`
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct BlockRef<'g> {
    value: IdRef<'g, BlockData<'g>>,
}

impl<'g> BlockRef<'g> {
    /// create a new reference to the passed in block
    pub fn new(value: IdRef<'g, BlockData<'g>>) -> Self {
        Self { value }
    }
    /// get the contained `IdRef<BlockData>`
    pub fn value(&self) -> IdRef<'g, BlockData<'g>> {
        self.value
    }
}

/// a block of code
#[derive(Eq, PartialEq, Hash, Debug)]
pub struct Block<'g> {
    value: IdRef<'g, BlockData<'g>>,
}

impl<'g> Block<'g> {
    /// create a new block of code.
    /// Sets the body if the `body` argument is `Some`.
    /// the name doesn't need to be unique
    pub fn new(
        name: impl Internable<'g, Interned = str>,
        body: Option<Vec<Instruction<'g>>>,
        result_definitions: Inhabitable<Vec<ValueDefinition<'g>>>,
        global_state: &'g GlobalState<'g>,
    ) -> Self {
        let body = match body {
            None => OnceCell::new(),
            Some(v) => OnceCell::from(v),
        };
        Block {
            value: global_state.alloc(BlockData {
                name: name.intern(global_state),
                body,
                result_definitions,
            }),
        }
    }
    /// create a new block of code, setting the body to the passed in value.
    /// the name doesn't need to be unique
    pub fn with_body(
        name: impl Internable<'g, Interned = str>,
        body: Vec<Instruction<'g>>,
        result_definitions: Inhabitable<Vec<ValueDefinition<'g>>>,
        global_state: &'g GlobalState<'g>,
    ) -> Self {
        Self::new(name, Some(body), result_definitions, global_state)
    }
    /// create a new block of code without setting the body.
    /// The body needs to be later set, `BlockData::set_body` can be used to do that.
    /// the name doesn't need to be unique
    pub fn without_body(
        name: impl Internable<'g, Interned = str>,
        result_definitions: Inhabitable<Vec<ValueDefinition<'g>>>,
        global_state: &'g GlobalState<'g>,
    ) -> Self {
        Self::new(name, None, result_definitions, global_state)
    }
    /// get the contained `IdRef<BlockData>`
    pub fn value(&self) -> IdRef<'g, BlockData<'g>> {
        self.value
    }
    /// the equivalent of `FromText::from_text` while additionally calling the
    /// provided callbacks at the appropriate points.
    fn from_text_with_callbacks<
        't,
        BeforeResultDefinitionsCallback: FnOnce(&mut FromTextState<'g, 't>) -> Result<(), FromTextError>,
        AfterResultDefinitionsCallback: FnOnce(&mut FromTextState<'g, 't>) -> Result<(), FromTextError>,
        BeforeBodyCallback: FnOnce(Self, &mut FromTextState<'g, 't>) -> Result<(), FromTextError>,
    >(
        state: &mut FromTextState<'g, 't>,
        before_result_definitions_callback: BeforeResultDefinitionsCallback,
        after_result_definitions_callback: AfterResultDefinitionsCallback,
        before_body_callback: BeforeBodyCallback,
    ) -> Result<IdRef<'g, BlockData<'g>>, FromTextError> {
        let kind_location = state.peek_token()?.span;
        if Self::KIND != InstructionKind::from_text(state)? {
            state.error_at(
                kind_location,
                format!("expected {} instruction", Self::KIND.text()),
            )?;
        }
        let name_location = state.peek_token()?.span;
        let name = NamedId::from_text(state)?;
        state.parse_punct_token_or_error(Punctuation::Arrow, "missing arrow: '->'")?;
        let initial_scope = state.scope_stack_top;
        before_result_definitions_callback(state)?;
        let result_definitions = Inhabitable::<Vec<ValueDefinition>>::from_text(state)?;
        after_result_definitions_callback(state)?;
        let results_scope = state.scope_stack_top;
        state.scope_stack_top = initial_scope;
        let scope = state.push_new_nested_scope();
        let block = Block::without_body(name.name, result_definitions, state.global_state());
        if state
            .insert_symbol(
                name,
                FromTextSymbol {
                    value: block.value(),
                    scope,
                },
            )
            .is_err()
        {
            state.error_at(name_location, "duplicate block name")?;
        }
        let missing_closing_brace = "missing closing curly brace: '}'";
        state.parse_parenthesized(
            Punctuation::LCurlyBrace,
            "missing opening curly brace: '{'",
            Punctuation::RCurlyBrace,
            missing_closing_brace,
            |state| -> Result<_, _> {
                let block_data = block.value();
                before_body_callback(block, state)?;
                let mut body = Vec::new();
                let mut end_reachable = true;
                loop {
                    let Token {
                        span: instruction_location,
                        kind: peek_token_kind,
                    } = state.peek_token()?;
                    match peek_token_kind {
                        TokenKind::EndOfFile => {
                            state.error_at_peek_token(missing_closing_brace)?;
                        }
                        TokenKind::Punct(Punctuation::RCurlyBrace) => break,
                        _ => {}
                    }
                    let instruction = Instruction::from_text(state)?;
                    state.parse_punct_token_or_error(
                        Punctuation::Semicolon,
                        "missing terminating semicolon: ';'",
                    )?;
                    if !end_reachable {
                        state.error_at(instruction_location, "unreachable instruction")?;
                    } else if let Uninhabited = instruction.results() {
                        end_reachable = false;
                    }
                    body.push(instruction);
                }
                if end_reachable {
                    state.error_at_peek_token("missing terminating instruction")?;
                }
                block_data.set_body(body);
                state.scope_stack_top = results_scope;
                Ok(block_data)
            },
        )
    }
}

impl<'g> Deref for BlockRef<'g> {
    type Target = IdRef<'g, BlockData<'g>>;
    fn deref(&self) -> &IdRef<'g, BlockData<'g>> {
        &self.value
    }
}

impl<'g> Deref for Block<'g> {
    type Target = IdRef<'g, BlockData<'g>>;
    fn deref(&self) -> &IdRef<'g, BlockData<'g>> {
        &self.value
    }
}

impl<'g> CodeIO<'g> for Block<'g> {
    fn results(&self) -> Inhabitable<&[ValueDefinition<'g>]> {
        self.value.results()
    }
    fn arguments(&self) -> &[ValueUse<'g>] {
        self.value.arguments()
    }
}

impl<'g> FromText<'g> for BlockRef<'g> {
    type Parsed = Self;
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError> {
        let name_location = state.peek_token()?.span;
        let name = NamedId::from_text(state)?;
        if let Some(FromTextSymbol { value, scope }) = state.get_symbol(name) {
            if state.is_scope_visible(scope) {
                Ok(BlockRef::new(value))
            } else {
                state.error_at(name_location, "block not in scope")?.into()
            }
        } else {
            state.error_at(name_location, "name not found")?.into()
        }
    }
}

impl<'g> ToText<'g> for BlockRef<'g> {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        match state.get_block_named_id(self.value()) {
            NewOrOld::New(_) => unreachable!("block instruction must be written first"),
            NewOrOld::Old(name) => name.to_text(state),
        }
    }
}

impl<'g> FromText<'g> for Block<'g> {
    type Parsed = Self;
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError> {
        let mut retval = None;
        Self::from_text_with_callbacks(
            state,
            |_state| Ok(()),
            |_state| Ok(()),
            |block, _state| {
                retval = Some(block);
                Ok(())
            },
        )?;
        Ok(retval.expect("known to be Some"))
    }
}

impl<'g> ToText<'g> for Block<'g> {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        write!(state, "{} ", Self::KIND.text())?;
        match state.get_block_named_id(self.value()) {
            NewOrOld::Old(_) => unreachable!("block instruction must be written first"),
            NewOrOld::New(name) => name.to_text(state)?,
        }
        write!(state, " -> ")?;
        let BlockData {
            name: _,
            body,
            result_definitions,
        } = &***self;
        result_definitions.to_text(state)?;
        writeln!(state, " {{")?;
        state.indent(|state| -> fmt::Result {
            for instruction in body.get().expect("block body not set") {
                instruction.to_text(state)?;
                writeln!(state, ";")?;
            }
            Ok(())
        })?;
        write!(state, "}}")
    }
}

/// the struct storing the data for a `Loop`
#[derive(Debug)]
pub struct LoopData<'g> {
    /// the name of the `Loop` -- doesn't need to be unique
    pub name: Interned<'g, str>,
    /// the values assigned to `self.header.argument_definitions` on the first iteration.
    /// The values assigned on later iterations are provided in the corresponding `ContinueLoop` instructions.
    pub arguments: Vec<ValueUse<'g>>,
    /// the loop header, holds the `ValueDefinition`s assigned at the beginning of each loop iteration
    pub header: LoopHeader<'g>,
    /// The body of the loop, the loop exits when `body` finishes.
    /// The values defined in `body.result_definitions` are the results of the loop.
    pub body: Block<'g>,
}

impl<'g> Id<'g> for LoopData<'g> {}

impl<'g> CodeIO<'g> for LoopData<'g> {
    fn results(&self) -> Inhabitable<&[ValueDefinition<'g>]> {
        self.body.results()
    }
    fn arguments(&self) -> &[ValueUse<'g>] {
        &self.arguments
    }
}

impl<'g> CodeIO<'g> for Loop<'g> {
    fn results(&self) -> Inhabitable<&[ValueDefinition<'g>]> {
        self.value.results()
    }
    fn arguments(&self) -> &[ValueUse<'g>] {
        self.value.arguments()
    }
}

/// a reference to a `Loop`
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct LoopRef<'g> {
    value: IdRef<'g, LoopData<'g>>,
}

impl<'g> LoopRef<'g> {
    /// create a new reference to the passed in loop
    pub fn new(value: IdRef<'g, LoopData<'g>>) -> Self {
        Self { value }
    }
    /// get the contained `IdRef<LoopData>`
    pub fn value(&self) -> IdRef<'g, LoopData<'g>> {
        self.value
    }
}

/// a loop
#[derive(Eq, PartialEq, Hash, Debug)]
pub struct Loop<'g> {
    value: IdRef<'g, LoopData<'g>>,
}

impl<'g> Loop<'g> {
    /// create a new `Loop`. the name doesn't need to be unique
    pub fn new(
        name: impl Internable<'g, Interned = str>,
        arguments: Vec<ValueUse<'g>>,
        argument_definitions: Vec<ValueDefinition<'g>>,
        body: Block<'g>,
        global_state: &'g GlobalState<'g>,
    ) -> Self {
        Loop {
            value: global_state.alloc(LoopData {
                name: name.intern(global_state),
                arguments,
                header: LoopHeader {
                    argument_definitions,
                },
                body,
            }),
        }
    }
    /// get the contained `IdRef<LoopData>`
    pub fn value(&self) -> IdRef<'g, LoopData<'g>> {
        self.value
    }
}

impl<'g> Deref for LoopRef<'g> {
    type Target = IdRef<'g, LoopData<'g>>;
    fn deref(&self) -> &IdRef<'g, LoopData<'g>> {
        &self.value
    }
}

impl<'g> Deref for Loop<'g> {
    type Target = IdRef<'g, LoopData<'g>>;
    fn deref(&self) -> &IdRef<'g, LoopData<'g>> {
        &self.value
    }
}

impl<'g> FromText<'g> for LoopRef<'g> {
    type Parsed = Self;
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError> {
        let name_location = state.peek_token()?.span;
        let name = NamedId::from_text(state)?;
        if let Some(FromTextSymbol { value, scope }) = state.get_symbol(name) {
            if state.is_scope_visible(scope) {
                Ok(LoopRef::new(value))
            } else {
                state.error_at(name_location, "loop not in scope")?.into()
            }
        } else {
            state.error_at(name_location, "name not found")?.into()
        }
    }
}

impl<'g> ToText<'g> for LoopRef<'g> {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        match state.get_loop_named_id(self.value()) {
            NewOrOld::New(_) => unreachable!("loop instruction must be written first"),
            NewOrOld::Old(name) => name.to_text(state),
        }
    }
}

impl<'g> FromText<'g> for Loop<'g> {
    type Parsed = Self;
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError> {
        let kind_location = state.peek_token()?.span;
        if Self::KIND != InstructionKind::from_text(state)? {
            state.error_at(
                kind_location,
                format!("expected {} instruction", Self::KIND.text()),
            )?;
        }
        let name_location = state.peek_token()?.span;
        let name = NamedId::from_text(state)?;
        let arguments = Vec::<ValueUse>::from_text(state)?;
        let missing_closing_brace = "missing closing curly brace: '}'";
        state.parse_parenthesized(
            Punctuation::LCurlyBrace,
            "missing opening curly brace: '{'",
            Punctuation::RCurlyBrace,
            missing_closing_brace,
            |state| -> Result<_, _> {
                let initial_scope = state.scope_stack_top;
                let scope = state.push_new_nested_scope();
                state.parse_punct_token_or_error(Punctuation::Arrow, "missing arrow: '->'")?;
                let argument_definitions = Vec::<ValueDefinition>::from_text(state)?;
                state.parse_punct_token_or_error(
                    Punctuation::Semicolon,
                    "missing terminating semicolon: ';'",
                )?;
                let mut retval = None;
                let mut results_scope = None;
                Block::from_text_with_callbacks(
                    state,
                    |state| {
                        state.scope_stack_top = initial_scope;
                        Ok(())
                    },
                    |state| {
                        results_scope = Some(state.scope_stack_top);
                        Ok(())
                    },
                    |block, state| {
                        let loop_ = Loop::new(
                            name.name,
                            arguments,
                            argument_definitions,
                            block,
                            state.global_state(),
                        );
                        if state
                            .insert_symbol(
                                name,
                                FromTextSymbol {
                                    value: loop_.value(),
                                    scope,
                                },
                            )
                            .is_err()
                        {
                            state.error_at(name_location, "duplicate loop name")?;
                        }
                        retval = Some(loop_);
                        Ok(())
                    },
                )?;
                state.scope_stack_top = results_scope.expect("known to be Some");
                Ok(retval.expect("known to be Some"))
            },
        )
    }
}

impl<'g> ToText<'g> for Loop<'g> {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        write!(state, "{} ", Self::KIND.text())?;
        match state.get_loop_named_id(self.value()) {
            NewOrOld::Old(_) => unreachable!("loop instruction must be written first"),
            NewOrOld::New(name) => name.to_text(state)?,
        }
        let LoopData {
            name: _,
            arguments,
            header: LoopHeader {
                argument_definitions,
            },
            body,
        } = &***self;
        write!(state, " ")?;
        arguments.to_text(state)?;
        writeln!(state, " {{")?;
        state.indent(|state| {
            write!(state, "-> ")?;
            argument_definitions.to_text(state)?;
            writeln!(state, ";")?;
            body.to_text(state)?;
            writeln!(state)
        })?;
        write!(state, "}}")
    }
}

/// continue a loop.
/// jumps back to the beginning of `self.target_loop`.
/// only valid when contained inside of `self.target_loop`.
#[derive(Debug)]
pub struct ContinueLoop<'g> {
    /// the loop to continue.
    pub target_loop: LoopRef<'g>,
    /// the values assigned to the loop header's `ValueDefinition`s: `self.target_loop.header.argument_definitions`.
    pub loop_arguments: Vec<ValueUse<'g>>,
}

impl<'g> ToText<'g> for ContinueLoop<'g> {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        Self::KIND.to_text(state)?;
        write!(state, " ")?;
        let Self {
            target_loop,
            loop_arguments,
        } = self;
        target_loop.to_text(state)?;
        loop_arguments.to_text(state)
    }
}

impl<'g> FromText<'g> for ContinueLoop<'g> {
    type Parsed = Self;
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError> {
        let kind_location = state.peek_token()?.span;
        if Self::KIND != InstructionKind::from_text(state)? {
            state.error_at(
                kind_location,
                format!("expected {} instruction", Self::KIND.text()),
            )?;
        }
        let target_loop = LoopRef::from_text(state)?;
        let loop_arguments = Vec::<ValueUse<'g>>::from_text(state)?;
        Ok(Self {
            target_loop,
            loop_arguments,
        })
    }
}

impl<'g> CodeIO<'g> for ContinueLoop<'g> {
    fn results(&self) -> Inhabitable<&[ValueDefinition<'g>]> {
        Uninhabited
    }
    fn arguments(&self) -> &[ValueUse<'g>] {
        &self.loop_arguments
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions;
    use crate::IntegerType;

    #[test]
    fn test_from_to_text() {
        let global_state = GlobalState::new();
        let global_state = &global_state;
        macro_rules! test_from_to_text {
            ($global_state:ident, $text:expr, $type:ty, $value:expr, $formatted_text:expr) => {{
                let text = $value.display().to_string();
                assert_eq!($formatted_text, text);
                let parsed_value = <$type>::parse("", $text, $global_state).unwrap();
                let text = parsed_value.display().to_string();
                assert_eq!($formatted_text, text);
            }};
            ($global_state:ident, $text:expr, $type:ty, $const:expr) => {
                test_from_to_text!($global_state, $text, $type, $const, $text);
            };
        }

        let block1 = Block::without_body("block1", Inhabited(vec![]), global_state);
        let mut block1_body = Vec::new();
        let add_result_def = ValueDefinition::new(IntegerType::Int32, "add_result", global_state);
        let add_result = add_result_def.value();
        block1_body.push(Instruction::without_location(instructions::Add {
            arguments: [
                ValueUse::from_const(1u32, "", global_state),
                ValueUse::from_const(2u32, "", global_state),
            ],
            results: [add_result_def],
        }));
        block1_body.push(Instruction::without_location(instructions::Branch {
            variable: ValueUse::new(add_result),
            targets: vec![instructions::BranchTarget {
                value: 3u32.intern(global_state),
                break_block: BreakBlock {
                    block: BlockRef::new(block1.value()),
                    block_results: vec![],
                },
            }],
        }));
        block1_body.push(Instruction::with_location(
            Location::new_interned("my_source.vertex", 123, 456, global_state),
            BreakBlock {
                block: BlockRef::new(block1.value()),
                block_results: vec![],
            },
        ));
        block1.set_body(block1_body);
        test_from_to_text!(
            global_state,
            concat!(
                "block block1 -> [] {\n",
                "    add [\"\"0 : 0x1i32, \"\"1 : 0x2i32] -> [add_result : i32];\n",
                "    branch [add_result], [0x3i32 -> break block1[]] -> [];\n",
                "    break block1[] @ \"my_source.vertex\":123:456;\n",
                "}"
            ),
            Block,
            block1
        );

        let block1 = Block::without_body("block1", Uninhabited, global_state);
        let mut block1_body = Vec::new();
        let block2 = Block::without_body("block2", Uninhabited, global_state);
        let mut block2_body = Vec::new();
        let loop1 = Loop::new("loop1", vec![], vec![], block2, global_state);
        block2_body.push(Instruction::without_location(ContinueLoop {
            target_loop: LoopRef::new(loop1.value()),
            loop_arguments: vec![],
        }));
        loop1.body.set_body(block2_body);
        block1_body.push(Instruction::without_location(loop1));
        block1.set_body(block1_body);
        test_from_to_text!(
            global_state,
            concat!(
                "block block1 -> ! {\n",
                "    loop loop1 [] {\n",
                "        -> [];\n",
                "        block block2 -> ! {\n",
                "            continue loop1[];\n",
                "        }\n",
                "    };\n",
                "}"
            ),
            Block,
            block1
        );
    }
}