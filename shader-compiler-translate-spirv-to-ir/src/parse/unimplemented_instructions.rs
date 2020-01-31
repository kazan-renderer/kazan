// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::{
    parse::{
        functions::TranslationStateParsingFunctionBody, ParseInstruction,
        TranslationStateParsingTypesConstantsAndGlobals,
    },
    TranslationResult,
};

macro_rules! unimplemented_instruction {
    ($opname:ident) => {
        impl ParseInstruction for spirv_parser::$opname {
            fn parse_in_types_constants_globals_section<'g, 'i>(
                &'i self,
                _state: &mut TranslationStateParsingTypesConstantsAndGlobals<'g, 'i>,
            ) -> TranslationResult<()> {
                todo!(concat!("unimplemented instruction: ", stringify!($opname)))
            }
            fn parse_in_function_body<'g, 'i>(
                &'i self,
                _state: &mut TranslationStateParsingFunctionBody<'g, 'i>,
            ) -> TranslationResult<()> {
                todo!(concat!("unimplemented instruction: ", stringify!($opname)))
            }
        }
    };
}

unimplemented_instruction!(OpNop);
unimplemented_instruction!(OpUndef);
unimplemented_instruction!(OpExtInst);
unimplemented_instruction!(OpFunctionCall);
unimplemented_instruction!(OpImageTexelPointer);
unimplemented_instruction!(OpLoad);
unimplemented_instruction!(OpStore);
unimplemented_instruction!(OpCopyMemory);
unimplemented_instruction!(OpCopyMemorySized);
unimplemented_instruction!(OpAccessChain);
unimplemented_instruction!(OpInBoundsAccessChain);
unimplemented_instruction!(OpPtrAccessChain);
unimplemented_instruction!(OpArrayLength);
unimplemented_instruction!(OpGenericPtrMemSemantics);
unimplemented_instruction!(OpInBoundsPtrAccessChain);
unimplemented_instruction!(OpVectorExtractDynamic);
unimplemented_instruction!(OpVectorInsertDynamic);
unimplemented_instruction!(OpVectorShuffle);
unimplemented_instruction!(OpCompositeConstruct);
unimplemented_instruction!(OpCompositeExtract);
unimplemented_instruction!(OpCompositeInsert);
unimplemented_instruction!(OpCopyObject);
unimplemented_instruction!(OpTranspose);
unimplemented_instruction!(OpSampledImage);
unimplemented_instruction!(OpImageSampleImplicitLod);
unimplemented_instruction!(OpImageSampleExplicitLod);
unimplemented_instruction!(OpImageSampleDrefImplicitLod);
unimplemented_instruction!(OpImageSampleDrefExplicitLod);
unimplemented_instruction!(OpImageSampleProjImplicitLod);
unimplemented_instruction!(OpImageSampleProjExplicitLod);
unimplemented_instruction!(OpImageSampleProjDrefImplicitLod);
unimplemented_instruction!(OpImageSampleProjDrefExplicitLod);
unimplemented_instruction!(OpImageFetch);
unimplemented_instruction!(OpImageGather);
unimplemented_instruction!(OpImageDrefGather);
unimplemented_instruction!(OpImageRead);
unimplemented_instruction!(OpImageWrite);
unimplemented_instruction!(OpImage);
unimplemented_instruction!(OpImageQueryFormat);
unimplemented_instruction!(OpImageQueryOrder);
unimplemented_instruction!(OpImageQuerySizeLod);
unimplemented_instruction!(OpImageQuerySize);
unimplemented_instruction!(OpImageQueryLod);
unimplemented_instruction!(OpImageQueryLevels);
unimplemented_instruction!(OpImageQuerySamples);
unimplemented_instruction!(OpConvertFToU);
unimplemented_instruction!(OpConvertFToS);
unimplemented_instruction!(OpConvertSToF);
unimplemented_instruction!(OpConvertUToF);
unimplemented_instruction!(OpUConvert);
unimplemented_instruction!(OpSConvert);
unimplemented_instruction!(OpFConvert);
unimplemented_instruction!(OpQuantizeToF16);
unimplemented_instruction!(OpConvertPtrToU);
unimplemented_instruction!(OpSatConvertSToU);
unimplemented_instruction!(OpSatConvertUToS);
unimplemented_instruction!(OpConvertUToPtr);
unimplemented_instruction!(OpPtrCastToGeneric);
unimplemented_instruction!(OpGenericCastToPtr);
unimplemented_instruction!(OpGenericCastToPtrExplicit);
unimplemented_instruction!(OpBitcast);
unimplemented_instruction!(OpSNegate);
unimplemented_instruction!(OpFNegate);
unimplemented_instruction!(OpIAdd);
unimplemented_instruction!(OpFAdd);
unimplemented_instruction!(OpISub);
unimplemented_instruction!(OpFSub);
unimplemented_instruction!(OpIMul);
unimplemented_instruction!(OpFMul);
unimplemented_instruction!(OpUDiv);
unimplemented_instruction!(OpSDiv);
unimplemented_instruction!(OpFDiv);
unimplemented_instruction!(OpUMod);
unimplemented_instruction!(OpSRem);
unimplemented_instruction!(OpSMod);
unimplemented_instruction!(OpFRem);
unimplemented_instruction!(OpFMod);
unimplemented_instruction!(OpVectorTimesScalar);
unimplemented_instruction!(OpMatrixTimesScalar);
unimplemented_instruction!(OpVectorTimesMatrix);
unimplemented_instruction!(OpMatrixTimesVector);
unimplemented_instruction!(OpMatrixTimesMatrix);
unimplemented_instruction!(OpOuterProduct);
unimplemented_instruction!(OpDot);
unimplemented_instruction!(OpIAddCarry);
unimplemented_instruction!(OpISubBorrow);
unimplemented_instruction!(OpUMulExtended);
unimplemented_instruction!(OpSMulExtended);
unimplemented_instruction!(OpAny);
unimplemented_instruction!(OpAll);
unimplemented_instruction!(OpIsNan);
unimplemented_instruction!(OpIsInf);
unimplemented_instruction!(OpIsFinite);
unimplemented_instruction!(OpIsNormal);
unimplemented_instruction!(OpSignBitSet);
unimplemented_instruction!(OpLessOrGreater);
unimplemented_instruction!(OpOrdered);
unimplemented_instruction!(OpUnordered);
unimplemented_instruction!(OpLogicalEqual);
unimplemented_instruction!(OpLogicalNotEqual);
unimplemented_instruction!(OpLogicalOr);
unimplemented_instruction!(OpLogicalAnd);
unimplemented_instruction!(OpLogicalNot);
unimplemented_instruction!(OpSelect);
unimplemented_instruction!(OpIEqual);
unimplemented_instruction!(OpINotEqual);
unimplemented_instruction!(OpUGreaterThan);
unimplemented_instruction!(OpSGreaterThan);
unimplemented_instruction!(OpUGreaterThanEqual);
unimplemented_instruction!(OpSGreaterThanEqual);
unimplemented_instruction!(OpULessThan);
unimplemented_instruction!(OpSLessThan);
unimplemented_instruction!(OpULessThanEqual);
unimplemented_instruction!(OpSLessThanEqual);
unimplemented_instruction!(OpFOrdEqual);
unimplemented_instruction!(OpFUnordEqual);
unimplemented_instruction!(OpFOrdNotEqual);
unimplemented_instruction!(OpFUnordNotEqual);
unimplemented_instruction!(OpFOrdLessThan);
unimplemented_instruction!(OpFUnordLessThan);
unimplemented_instruction!(OpFOrdGreaterThan);
unimplemented_instruction!(OpFUnordGreaterThan);
unimplemented_instruction!(OpFOrdLessThanEqual);
unimplemented_instruction!(OpFUnordLessThanEqual);
unimplemented_instruction!(OpFOrdGreaterThanEqual);
unimplemented_instruction!(OpFUnordGreaterThanEqual);
unimplemented_instruction!(OpShiftRightLogical);
unimplemented_instruction!(OpShiftRightArithmetic);
unimplemented_instruction!(OpShiftLeftLogical);
unimplemented_instruction!(OpBitwiseOr);
unimplemented_instruction!(OpBitwiseXor);
unimplemented_instruction!(OpBitwiseAnd);
unimplemented_instruction!(OpNot);
unimplemented_instruction!(OpBitFieldInsert);
unimplemented_instruction!(OpBitFieldSExtract);
unimplemented_instruction!(OpBitFieldUExtract);
unimplemented_instruction!(OpBitReverse);
unimplemented_instruction!(OpBitCount);
unimplemented_instruction!(OpDPdx);
unimplemented_instruction!(OpDPdy);
unimplemented_instruction!(OpFwidth);
unimplemented_instruction!(OpDPdxFine);
unimplemented_instruction!(OpDPdyFine);
unimplemented_instruction!(OpFwidthFine);
unimplemented_instruction!(OpDPdxCoarse);
unimplemented_instruction!(OpDPdyCoarse);
unimplemented_instruction!(OpFwidthCoarse);
unimplemented_instruction!(OpEmitVertex);
unimplemented_instruction!(OpEndPrimitive);
unimplemented_instruction!(OpEmitStreamVertex);
unimplemented_instruction!(OpEndStreamPrimitive);
unimplemented_instruction!(OpControlBarrier);
unimplemented_instruction!(OpMemoryBarrier);
unimplemented_instruction!(OpAtomicLoad);
unimplemented_instruction!(OpAtomicStore);
unimplemented_instruction!(OpAtomicExchange);
unimplemented_instruction!(OpAtomicCompareExchange);
unimplemented_instruction!(OpAtomicCompareExchangeWeak);
unimplemented_instruction!(OpAtomicIIncrement);
unimplemented_instruction!(OpAtomicIDecrement);
unimplemented_instruction!(OpAtomicIAdd);
unimplemented_instruction!(OpAtomicISub);
unimplemented_instruction!(OpAtomicSMin);
unimplemented_instruction!(OpAtomicUMin);
unimplemented_instruction!(OpAtomicSMax);
unimplemented_instruction!(OpAtomicUMax);
unimplemented_instruction!(OpAtomicAnd);
unimplemented_instruction!(OpAtomicOr);
unimplemented_instruction!(OpAtomicXor);
unimplemented_instruction!(OpPhi);
unimplemented_instruction!(OpLoopMerge);
unimplemented_instruction!(OpSelectionMerge);
unimplemented_instruction!(OpLabel);
unimplemented_instruction!(OpBranch);
unimplemented_instruction!(OpBranchConditional);
unimplemented_instruction!(OpSwitch32);
unimplemented_instruction!(OpSwitch64);
unimplemented_instruction!(OpKill);
unimplemented_instruction!(OpReturn);
unimplemented_instruction!(OpReturnValue);
unimplemented_instruction!(OpUnreachable);
unimplemented_instruction!(OpLifetimeStart);
unimplemented_instruction!(OpLifetimeStop);
unimplemented_instruction!(OpGroupAsyncCopy);
unimplemented_instruction!(OpGroupWaitEvents);
unimplemented_instruction!(OpGroupAll);
unimplemented_instruction!(OpGroupAny);
unimplemented_instruction!(OpGroupBroadcast);
unimplemented_instruction!(OpGroupIAdd);
unimplemented_instruction!(OpGroupFAdd);
unimplemented_instruction!(OpGroupFMin);
unimplemented_instruction!(OpGroupUMin);
unimplemented_instruction!(OpGroupSMin);
unimplemented_instruction!(OpGroupFMax);
unimplemented_instruction!(OpGroupUMax);
unimplemented_instruction!(OpGroupSMax);
unimplemented_instruction!(OpReadPipe);
unimplemented_instruction!(OpWritePipe);
unimplemented_instruction!(OpReservedReadPipe);
unimplemented_instruction!(OpReservedWritePipe);
unimplemented_instruction!(OpReserveReadPipePackets);
unimplemented_instruction!(OpReserveWritePipePackets);
unimplemented_instruction!(OpCommitReadPipe);
unimplemented_instruction!(OpCommitWritePipe);
unimplemented_instruction!(OpIsValidReserveId);
unimplemented_instruction!(OpGetNumPipePackets);
unimplemented_instruction!(OpGetMaxPipePackets);
unimplemented_instruction!(OpGroupReserveReadPipePackets);
unimplemented_instruction!(OpGroupReserveWritePipePackets);
unimplemented_instruction!(OpGroupCommitReadPipe);
unimplemented_instruction!(OpGroupCommitWritePipe);
unimplemented_instruction!(OpEnqueueMarker);
unimplemented_instruction!(OpEnqueueKernel);
unimplemented_instruction!(OpGetKernelNDrangeSubGroupCount);
unimplemented_instruction!(OpGetKernelNDrangeMaxSubGroupSize);
unimplemented_instruction!(OpGetKernelWorkGroupSize);
unimplemented_instruction!(OpGetKernelPreferredWorkGroupSizeMultiple);
unimplemented_instruction!(OpRetainEvent);
unimplemented_instruction!(OpReleaseEvent);
unimplemented_instruction!(OpCreateUserEvent);
unimplemented_instruction!(OpIsValidEvent);
unimplemented_instruction!(OpSetUserEventStatus);
unimplemented_instruction!(OpCaptureEventProfilingInfo);
unimplemented_instruction!(OpGetDefaultQueue);
unimplemented_instruction!(OpBuildNDRange);
unimplemented_instruction!(OpImageSparseSampleImplicitLod);
unimplemented_instruction!(OpImageSparseSampleExplicitLod);
unimplemented_instruction!(OpImageSparseSampleDrefImplicitLod);
unimplemented_instruction!(OpImageSparseSampleDrefExplicitLod);
unimplemented_instruction!(OpImageSparseFetch);
unimplemented_instruction!(OpImageSparseGather);
unimplemented_instruction!(OpImageSparseDrefGather);
unimplemented_instruction!(OpImageSparseTexelsResident);
unimplemented_instruction!(OpAtomicFlagTestAndSet);
unimplemented_instruction!(OpAtomicFlagClear);
unimplemented_instruction!(OpImageSparseRead);
unimplemented_instruction!(OpSizeOf);
unimplemented_instruction!(OpCreatePipeFromPipeStorage);
unimplemented_instruction!(OpGetKernelLocalSizeForSubgroupCount);
unimplemented_instruction!(OpGetKernelMaxNumSubgroups);
unimplemented_instruction!(OpNamedBarrierInitialize);
unimplemented_instruction!(OpMemoryNamedBarrier);
unimplemented_instruction!(OpGroupNonUniformElect);
unimplemented_instruction!(OpGroupNonUniformAll);
unimplemented_instruction!(OpGroupNonUniformAny);
unimplemented_instruction!(OpGroupNonUniformAllEqual);
unimplemented_instruction!(OpGroupNonUniformBroadcast);
unimplemented_instruction!(OpGroupNonUniformBroadcastFirst);
unimplemented_instruction!(OpGroupNonUniformBallot);
unimplemented_instruction!(OpGroupNonUniformInverseBallot);
unimplemented_instruction!(OpGroupNonUniformBallotBitExtract);
unimplemented_instruction!(OpGroupNonUniformBallotBitCount);
unimplemented_instruction!(OpGroupNonUniformBallotFindLSB);
unimplemented_instruction!(OpGroupNonUniformBallotFindMSB);
unimplemented_instruction!(OpGroupNonUniformShuffle);
unimplemented_instruction!(OpGroupNonUniformShuffleXor);
unimplemented_instruction!(OpGroupNonUniformShuffleUp);
unimplemented_instruction!(OpGroupNonUniformShuffleDown);
unimplemented_instruction!(OpGroupNonUniformIAdd);
unimplemented_instruction!(OpGroupNonUniformFAdd);
unimplemented_instruction!(OpGroupNonUniformIMul);
unimplemented_instruction!(OpGroupNonUniformFMul);
unimplemented_instruction!(OpGroupNonUniformSMin);
unimplemented_instruction!(OpGroupNonUniformUMin);
unimplemented_instruction!(OpGroupNonUniformFMin);
unimplemented_instruction!(OpGroupNonUniformSMax);
unimplemented_instruction!(OpGroupNonUniformUMax);
unimplemented_instruction!(OpGroupNonUniformFMax);
unimplemented_instruction!(OpGroupNonUniformBitwiseAnd);
unimplemented_instruction!(OpGroupNonUniformBitwiseOr);
unimplemented_instruction!(OpGroupNonUniformBitwiseXor);
unimplemented_instruction!(OpGroupNonUniformLogicalAnd);
unimplemented_instruction!(OpGroupNonUniformLogicalOr);
unimplemented_instruction!(OpGroupNonUniformLogicalXor);
unimplemented_instruction!(OpGroupNonUniformQuadBroadcast);
unimplemented_instruction!(OpGroupNonUniformQuadSwap);
unimplemented_instruction!(OpCopyLogical);
unimplemented_instruction!(OpPtrEqual);
unimplemented_instruction!(OpPtrNotEqual);
unimplemented_instruction!(OpPtrDiff);
unimplemented_instruction!(OpGLSLStd450Round);
unimplemented_instruction!(OpGLSLStd450RoundEven);
unimplemented_instruction!(OpGLSLStd450Trunc);
unimplemented_instruction!(OpGLSLStd450FAbs);
unimplemented_instruction!(OpGLSLStd450SAbs);
unimplemented_instruction!(OpGLSLStd450FSign);
unimplemented_instruction!(OpGLSLStd450SSign);
unimplemented_instruction!(OpGLSLStd450Floor);
unimplemented_instruction!(OpGLSLStd450Ceil);
unimplemented_instruction!(OpGLSLStd450Fract);
unimplemented_instruction!(OpGLSLStd450Radians);
unimplemented_instruction!(OpGLSLStd450Degrees);
unimplemented_instruction!(OpGLSLStd450Sin);
unimplemented_instruction!(OpGLSLStd450Cos);
unimplemented_instruction!(OpGLSLStd450Tan);
unimplemented_instruction!(OpGLSLStd450Asin);
unimplemented_instruction!(OpGLSLStd450Acos);
unimplemented_instruction!(OpGLSLStd450Atan);
unimplemented_instruction!(OpGLSLStd450Sinh);
unimplemented_instruction!(OpGLSLStd450Cosh);
unimplemented_instruction!(OpGLSLStd450Tanh);
unimplemented_instruction!(OpGLSLStd450Asinh);
unimplemented_instruction!(OpGLSLStd450Acosh);
unimplemented_instruction!(OpGLSLStd450Atanh);
unimplemented_instruction!(OpGLSLStd450Atan2);
unimplemented_instruction!(OpGLSLStd450Pow);
unimplemented_instruction!(OpGLSLStd450Exp);
unimplemented_instruction!(OpGLSLStd450Log);
unimplemented_instruction!(OpGLSLStd450Exp2);
unimplemented_instruction!(OpGLSLStd450Log2);
unimplemented_instruction!(OpGLSLStd450Sqrt);
unimplemented_instruction!(OpGLSLStd450InverseSqrt);
unimplemented_instruction!(OpGLSLStd450Determinant);
unimplemented_instruction!(OpGLSLStd450MatrixInverse);
unimplemented_instruction!(OpGLSLStd450Modf);
unimplemented_instruction!(OpGLSLStd450ModfStruct);
unimplemented_instruction!(OpGLSLStd450FMin);
unimplemented_instruction!(OpGLSLStd450UMin);
unimplemented_instruction!(OpGLSLStd450SMin);
unimplemented_instruction!(OpGLSLStd450FMax);
unimplemented_instruction!(OpGLSLStd450UMax);
unimplemented_instruction!(OpGLSLStd450SMax);
unimplemented_instruction!(OpGLSLStd450FClamp);
unimplemented_instruction!(OpGLSLStd450UClamp);
unimplemented_instruction!(OpGLSLStd450SClamp);
unimplemented_instruction!(OpGLSLStd450FMix);
unimplemented_instruction!(OpGLSLStd450IMix);
unimplemented_instruction!(OpGLSLStd450Step);
unimplemented_instruction!(OpGLSLStd450SmoothStep);
unimplemented_instruction!(OpGLSLStd450Fma);
unimplemented_instruction!(OpGLSLStd450Frexp);
unimplemented_instruction!(OpGLSLStd450FrexpStruct);
unimplemented_instruction!(OpGLSLStd450Ldexp);
unimplemented_instruction!(OpGLSLStd450PackSnorm4x8);
unimplemented_instruction!(OpGLSLStd450PackUnorm4x8);
unimplemented_instruction!(OpGLSLStd450PackSnorm2x16);
unimplemented_instruction!(OpGLSLStd450PackUnorm2x16);
unimplemented_instruction!(OpGLSLStd450PackHalf2x16);
unimplemented_instruction!(OpGLSLStd450PackDouble2x32);
unimplemented_instruction!(OpGLSLStd450UnpackSnorm2x16);
unimplemented_instruction!(OpGLSLStd450UnpackUnorm2x16);
unimplemented_instruction!(OpGLSLStd450UnpackHalf2x16);
unimplemented_instruction!(OpGLSLStd450UnpackSnorm4x8);
unimplemented_instruction!(OpGLSLStd450UnpackUnorm4x8);
unimplemented_instruction!(OpGLSLStd450UnpackDouble2x32);
unimplemented_instruction!(OpGLSLStd450Length);
unimplemented_instruction!(OpGLSLStd450Distance);
unimplemented_instruction!(OpGLSLStd450Cross);
unimplemented_instruction!(OpGLSLStd450Normalize);
unimplemented_instruction!(OpGLSLStd450FaceForward);
unimplemented_instruction!(OpGLSLStd450Reflect);
unimplemented_instruction!(OpGLSLStd450Refract);
unimplemented_instruction!(OpGLSLStd450FindILsb);
unimplemented_instruction!(OpGLSLStd450FindSMsb);
unimplemented_instruction!(OpGLSLStd450FindUMsb);
unimplemented_instruction!(OpGLSLStd450InterpolateAtCentroid);
unimplemented_instruction!(OpGLSLStd450InterpolateAtSample);
unimplemented_instruction!(OpGLSLStd450InterpolateAtOffset);
unimplemented_instruction!(OpGLSLStd450NMin);
unimplemented_instruction!(OpGLSLStd450NMax);
unimplemented_instruction!(OpGLSLStd450NClamp);
unimplemented_instruction!(OpOpenCLStdAcos);
unimplemented_instruction!(OpOpenCLStdAcosh);
unimplemented_instruction!(OpOpenCLStdAcospi);
unimplemented_instruction!(OpOpenCLStdAsin);
unimplemented_instruction!(OpOpenCLStdAsinh);
unimplemented_instruction!(OpOpenCLStdAsinpi);
unimplemented_instruction!(OpOpenCLStdAtan);
unimplemented_instruction!(OpOpenCLStdAtan2);
unimplemented_instruction!(OpOpenCLStdAtanh);
unimplemented_instruction!(OpOpenCLStdAtanpi);
unimplemented_instruction!(OpOpenCLStdAtan2pi);
unimplemented_instruction!(OpOpenCLStdCbrt);
unimplemented_instruction!(OpOpenCLStdCeil);
unimplemented_instruction!(OpOpenCLStdCopysign);
unimplemented_instruction!(OpOpenCLStdCos);
unimplemented_instruction!(OpOpenCLStdCosh);
unimplemented_instruction!(OpOpenCLStdCospi);
unimplemented_instruction!(OpOpenCLStdErfc);
unimplemented_instruction!(OpOpenCLStdErf);
unimplemented_instruction!(OpOpenCLStdExp);
unimplemented_instruction!(OpOpenCLStdExp2);
unimplemented_instruction!(OpOpenCLStdExp10);
unimplemented_instruction!(OpOpenCLStdExpm1);
unimplemented_instruction!(OpOpenCLStdFabs);
unimplemented_instruction!(OpOpenCLStdFdim);
unimplemented_instruction!(OpOpenCLStdFloor);
unimplemented_instruction!(OpOpenCLStdFma);
unimplemented_instruction!(OpOpenCLStdFmax);
unimplemented_instruction!(OpOpenCLStdFmin);
unimplemented_instruction!(OpOpenCLStdFmod);
unimplemented_instruction!(OpOpenCLStdFract);
unimplemented_instruction!(OpOpenCLStdFrexp);
unimplemented_instruction!(OpOpenCLStdHypot);
unimplemented_instruction!(OpOpenCLStdIlogb);
unimplemented_instruction!(OpOpenCLStdLdexp);
unimplemented_instruction!(OpOpenCLStdLgamma);
unimplemented_instruction!(OpOpenCLStdLgammaR);
unimplemented_instruction!(OpOpenCLStdLog);
unimplemented_instruction!(OpOpenCLStdLog2);
unimplemented_instruction!(OpOpenCLStdLog10);
unimplemented_instruction!(OpOpenCLStdLog1p);
unimplemented_instruction!(OpOpenCLStdLogb);
unimplemented_instruction!(OpOpenCLStdMad);
unimplemented_instruction!(OpOpenCLStdMaxmag);
unimplemented_instruction!(OpOpenCLStdMinmag);
unimplemented_instruction!(OpOpenCLStdModf);
unimplemented_instruction!(OpOpenCLStdNan);
unimplemented_instruction!(OpOpenCLStdNextafter);
unimplemented_instruction!(OpOpenCLStdPow);
unimplemented_instruction!(OpOpenCLStdPown);
unimplemented_instruction!(OpOpenCLStdPowr);
unimplemented_instruction!(OpOpenCLStdRemainder);
unimplemented_instruction!(OpOpenCLStdRemquo);
unimplemented_instruction!(OpOpenCLStdRint);
unimplemented_instruction!(OpOpenCLStdRootn);
unimplemented_instruction!(OpOpenCLStdRound);
unimplemented_instruction!(OpOpenCLStdRsqrt);
unimplemented_instruction!(OpOpenCLStdSin);
unimplemented_instruction!(OpOpenCLStdSincos);
unimplemented_instruction!(OpOpenCLStdSinh);
unimplemented_instruction!(OpOpenCLStdSinpi);
unimplemented_instruction!(OpOpenCLStdSqrt);
unimplemented_instruction!(OpOpenCLStdTan);
unimplemented_instruction!(OpOpenCLStdTanh);
unimplemented_instruction!(OpOpenCLStdTanpi);
unimplemented_instruction!(OpOpenCLStdTgamma);
unimplemented_instruction!(OpOpenCLStdTrunc);
unimplemented_instruction!(OpOpenCLStdHalfCos);
unimplemented_instruction!(OpOpenCLStdHalfDivide);
unimplemented_instruction!(OpOpenCLStdHalfExp);
unimplemented_instruction!(OpOpenCLStdHalfExp2);
unimplemented_instruction!(OpOpenCLStdHalfExp10);
unimplemented_instruction!(OpOpenCLStdHalfLog);
unimplemented_instruction!(OpOpenCLStdHalfLog2);
unimplemented_instruction!(OpOpenCLStdHalfLog10);
unimplemented_instruction!(OpOpenCLStdHalfPowr);
unimplemented_instruction!(OpOpenCLStdHalfRecip);
unimplemented_instruction!(OpOpenCLStdHalfRsqrt);
unimplemented_instruction!(OpOpenCLStdHalfSin);
unimplemented_instruction!(OpOpenCLStdHalfSqrt);
unimplemented_instruction!(OpOpenCLStdHalfTan);
unimplemented_instruction!(OpOpenCLStdNativeCos);
unimplemented_instruction!(OpOpenCLStdNativeDivide);
unimplemented_instruction!(OpOpenCLStdNativeExp);
unimplemented_instruction!(OpOpenCLStdNativeExp2);
unimplemented_instruction!(OpOpenCLStdNativeExp10);
unimplemented_instruction!(OpOpenCLStdNativeLog);
unimplemented_instruction!(OpOpenCLStdNativeLog2);
unimplemented_instruction!(OpOpenCLStdNativeLog10);
unimplemented_instruction!(OpOpenCLStdNativePowr);
unimplemented_instruction!(OpOpenCLStdNativeRecip);
unimplemented_instruction!(OpOpenCLStdNativeRsqrt);
unimplemented_instruction!(OpOpenCLStdNativeSin);
unimplemented_instruction!(OpOpenCLStdNativeSqrt);
unimplemented_instruction!(OpOpenCLStdNativeTan);
unimplemented_instruction!(OpOpenCLStdSAbs);
unimplemented_instruction!(OpOpenCLStdSAbsDiff);
unimplemented_instruction!(OpOpenCLStdSAddSat);
unimplemented_instruction!(OpOpenCLStdUAddSat);
unimplemented_instruction!(OpOpenCLStdSHadd);
unimplemented_instruction!(OpOpenCLStdUHadd);
unimplemented_instruction!(OpOpenCLStdSRhadd);
unimplemented_instruction!(OpOpenCLStdURhadd);
unimplemented_instruction!(OpOpenCLStdSClamp);
unimplemented_instruction!(OpOpenCLStdUClamp);
unimplemented_instruction!(OpOpenCLStdClz);
unimplemented_instruction!(OpOpenCLStdCtz);
unimplemented_instruction!(OpOpenCLStdSMadHi);
unimplemented_instruction!(OpOpenCLStdUMadSat);
unimplemented_instruction!(OpOpenCLStdSMadSat);
unimplemented_instruction!(OpOpenCLStdSMax);
unimplemented_instruction!(OpOpenCLStdUMax);
unimplemented_instruction!(OpOpenCLStdSMin);
unimplemented_instruction!(OpOpenCLStdUMin);
unimplemented_instruction!(OpOpenCLStdSMulHi);
unimplemented_instruction!(OpOpenCLStdRotate);
unimplemented_instruction!(OpOpenCLStdSSubSat);
unimplemented_instruction!(OpOpenCLStdUSubSat);
unimplemented_instruction!(OpOpenCLStdUUpsample);
unimplemented_instruction!(OpOpenCLStdSUpsample);
unimplemented_instruction!(OpOpenCLStdPopcount);
unimplemented_instruction!(OpOpenCLStdSMad24);
unimplemented_instruction!(OpOpenCLStdUMad24);
unimplemented_instruction!(OpOpenCLStdSMul24);
unimplemented_instruction!(OpOpenCLStdUMul24);
unimplemented_instruction!(OpOpenCLStdUAbs);
unimplemented_instruction!(OpOpenCLStdUAbsDiff);
unimplemented_instruction!(OpOpenCLStdUMulHi);
unimplemented_instruction!(OpOpenCLStdUMadHi);
unimplemented_instruction!(OpOpenCLStdFclamp);
unimplemented_instruction!(OpOpenCLStdDegrees);
unimplemented_instruction!(OpOpenCLStdFmaxCommon);
unimplemented_instruction!(OpOpenCLStdFminCommon);
unimplemented_instruction!(OpOpenCLStdMix);
unimplemented_instruction!(OpOpenCLStdRadians);
unimplemented_instruction!(OpOpenCLStdStep);
unimplemented_instruction!(OpOpenCLStdSmoothstep);
unimplemented_instruction!(OpOpenCLStdSign);
unimplemented_instruction!(OpOpenCLStdCross);
unimplemented_instruction!(OpOpenCLStdDistance);
unimplemented_instruction!(OpOpenCLStdLength);
unimplemented_instruction!(OpOpenCLStdNormalize);
unimplemented_instruction!(OpOpenCLStdFastDistance);
unimplemented_instruction!(OpOpenCLStdFastLength);
unimplemented_instruction!(OpOpenCLStdFastNormalize);
unimplemented_instruction!(OpOpenCLStdBitselect);
unimplemented_instruction!(OpOpenCLStdSelect);
unimplemented_instruction!(OpOpenCLStdVloadn);
unimplemented_instruction!(OpOpenCLStdVstoren);
unimplemented_instruction!(OpOpenCLStdVloadHalf);
unimplemented_instruction!(OpOpenCLStdVloadHalfn);
unimplemented_instruction!(OpOpenCLStdVstoreHalf);
unimplemented_instruction!(OpOpenCLStdVstoreHalfR);
unimplemented_instruction!(OpOpenCLStdVstoreHalfn);
unimplemented_instruction!(OpOpenCLStdVstoreHalfnR);
unimplemented_instruction!(OpOpenCLStdVloadaHalfn);
unimplemented_instruction!(OpOpenCLStdVstoreaHalfn);
unimplemented_instruction!(OpOpenCLStdVstoreaHalfnR);
unimplemented_instruction!(OpOpenCLStdShuffle);
unimplemented_instruction!(OpOpenCLStdShuffle2);
unimplemented_instruction!(OpOpenCLStdPrintf);
unimplemented_instruction!(OpOpenCLStdPrefetch);
