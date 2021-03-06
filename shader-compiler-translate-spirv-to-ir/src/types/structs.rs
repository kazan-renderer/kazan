// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::{
    decorations::{MemoryObjectDeclarationOrStructMember, VariableOrStructMember},
    errors::{
        BlockStructTypeNotAllowedAsMemberOfNonBlockStruct,
        InvalidComponentDecorationOnVariableOrStructMember, MemberDecorationNotAllowed,
        MissingLocationDecorationOnVariableOrStructMember, TranslationResult,
        TypeNotAllowedInUserDefinedVariableInterface,
    },
    io_layout::{io_interface_block_alignment, LOCATION_SIZE_IN_BYTES},
    types::{GenericSPIRVType, GetIrTypeState, IOInterfaceIR, IOInterfaceIRResult, SPIRVType},
};
use alloc::{rc::Rc, vec::Vec};
use core::ops::Deref;
use shader_compiler_ir::{
    Alignment, GlobalState, InterfaceBlockMember, Internable, Interned, StructSize,
    TargetProperties, UserInterfaceVariableAttributes,
};
use spirv_parser::{BuiltIn, DecorationLocation, IdResult, OpTypeStruct};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) enum StructKind {
    Generic,
    Block { is_buffer_block: bool },
    BuiltIns,
}

#[derive(Debug)]
pub(crate) struct StructMember<'g> {
    pub(crate) built_in: Option<BuiltIn>,
    pub(crate) member_type: SPIRVType<'g>,
    pub(crate) member_type_id: spirv_parser::IdRef,
    pub(crate) memory_object_declaration_or_struct_member: MemoryObjectDeclarationOrStructMember,
    pub(crate) variable_or_struct_member: VariableOrStructMember,
}

#[derive(Debug)]
pub(crate) struct StructTypeData<'g> {
    pub(crate) id: spirv_parser::IdRef,
    pub(crate) kind: StructKind,
    pub(crate) members: Vec<StructMember<'g>>,
}

impl<'g> StructTypeData<'g> {
    pub(crate) fn get_struct_instruction(&self) -> OpTypeStruct {
        OpTypeStruct {
            id_result: IdResult(self.id),
            member_types: self.members.iter().map(|v| v.member_type_id).collect(),
        }
    }
}

impl PartialEq<StructTypeData<'_>> for StructTypeData<'_> {
    fn eq(&self, rhs: &StructTypeData<'_>) -> bool {
        self.id == rhs.id
    }
}

impl Eq for StructTypeData<'_> {}

#[derive(Clone, Eq, PartialEq, Debug)]
pub(crate) struct StructType<'g>(Rc<StructTypeData<'g>>);

impl<'g> StructType<'g> {
    pub(crate) fn new(v: StructTypeData<'g>) -> Self {
        // TODO: assert validity
        Self(Rc::new(v))
    }
}

impl<'g> Deref for StructType<'g> {
    type Target = StructTypeData<'g>;
    fn deref(&self) -> &StructTypeData<'g> {
        &self.0
    }
}

impl<'g> From<StructType<'g>> for SPIRVType<'g> {
    fn from(v: StructType<'g>) -> Self {
        Self::Struct(v)
    }
}

impl<'g> GenericSPIRVType<'g> for StructType<'g> {
    fn get_ir_type_with_state(
        &self,
        _state: &mut GetIrTypeState<'g>,
    ) -> TranslationResult<Option<Interned<'g, shader_compiler_ir::Type<'g>>>> {
        todo!()
    }
    fn get_relaxed_precision_type(&self) -> Option<SPIRVType<'g>> {
        None
    }
    fn get_alignment<I: FnOnce() -> spirv_parser::Instruction>(
        &self,
        _target_properties: Interned<'g, TargetProperties>,
        _global_state: &'g GlobalState<'g>,
        _type_id: spirv_parser::IdRef,
        _instruction: I,
    ) -> TranslationResult<Alignment> {
        todo!()
    }
    fn translate_io_interface_to_ir(
        &self,
        global_state: &'g GlobalState<'g>,
        type_id: spirv_parser::IdRef,
        start_location: Option<u32>,
        start_io_component: Option<u32>,
    ) -> TranslationResult<IOInterfaceIRResult<'g>> {
        if let Some(component) = start_io_component {
            return Err(
                InvalidComponentDecorationOnVariableOrStructMember { type_id, component }.into(),
            );
        }
        let mut struct_size_in_bytes = 0;
        match self.kind {
            StructKind::Generic => {
                let mut next_location = start_location
                    .ok_or_else(|| MissingLocationDecorationOnVariableOrStructMember { type_id })?;
                let struct_byte_offset = next_location * LOCATION_SIZE_IN_BYTES;
                let mut members = Vec::with_capacity(self.members.len());
                for (member_index, member) in self.members.iter().enumerate() {
                    if let Some(location) = member.variable_or_struct_member.location {
                        return Err(MemberDecorationNotAllowed {
                            decoration: DecorationLocation { location }.into(),
                            member_index: member_index as u32,
                            instruction: self.get_struct_instruction().into(),
                        }
                        .into());
                    }
                    let IOInterfaceIRResult {
                        byte_offset,
                        size_in_bytes,
                        first_location_after,
                        ir,
                    } = member.member_type.translate_io_interface_to_ir(
                        global_state,
                        member.member_type_id,
                        Some(next_location),
                        member.memory_object_declaration_or_struct_member.component,
                    )?;
                    let member_type = match ir {
                        IOInterfaceIR::IRType(v) => v,
                        IOInterfaceIR::UserInterfaceBlockMembers(_)
                        | IOInterfaceIR::BuiltInInterfaceVariables(_) => {
                            return Err(BlockStructTypeNotAllowedAsMemberOfNonBlockStruct {
                                member_type_id: member.member_type_id,
                                outer_type_id: type_id,
                            }
                            .into())
                        }
                    };
                    next_location =
                        first_location_after.expect("first_location_after is known to be Some");
                    let offset = byte_offset
                        .checked_sub(struct_byte_offset)
                        .expect("invalid byte offset for struct member");
                    struct_size_in_bytes = struct_size_in_bytes.max(offset + size_in_bytes);
                    members.push(shader_compiler_ir::StructMember {
                        member_type,
                        offset,
                    });
                }
                Ok(IOInterfaceIRResult {
                    byte_offset: struct_byte_offset,
                    size_in_bytes: struct_size_in_bytes,
                    first_location_after: Some(next_location),
                    ir: shader_compiler_ir::StructType {
                        alignment: io_interface_block_alignment(),
                        size: StructSize::Fixed {
                            size: struct_size_in_bytes,
                        },
                        members,
                    }
                    .intern(global_state)
                    .into(),
                })
            }
            StructKind::Block { .. } => {
                let mut next_location = start_location;
                let mut members = Vec::with_capacity(self.members.len());
                for (member_index, member) in self.members.iter().enumerate() {
                    let member_start_location =
                        member.variable_or_struct_member.location.or(next_location);
                    let IOInterfaceIRResult {
                        byte_offset,
                        size_in_bytes,
                        first_location_after,
                        ir,
                    } = member.member_type.translate_io_interface_to_ir(
                        global_state,
                        member.member_type_id,
                        member_start_location,
                        member.memory_object_declaration_or_struct_member.component,
                    )?;
                    next_location = first_location_after;
                    struct_size_in_bytes = struct_size_in_bytes.max(byte_offset + size_in_bytes);
                    members.append(&mut ir.into_user_interface_block_members(
                        byte_offset,
                        || TypeNotAllowedInUserDefinedVariableInterface {
                            type_id: member.member_type_id,
                        },
                    )?);
                }
                Ok(IOInterfaceIRResult {
                    byte_offset: 0,
                    size_in_bytes: struct_size_in_bytes,
                    first_location_after: next_location,
                    ir: IOInterfaceIR::UserInterfaceBlockMembers(members),
                })
            }
            StructKind::BuiltIns => todo!(),
        }
    }
}
