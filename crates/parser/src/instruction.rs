use wasmparser::{BinaryReader, VisitOperator};

use crate::decode::Decoder;
use crate::error::Error;
use crate::types::{BlockType, HeapType, RefType, ValType};

#[derive(Clone, Debug, PartialEq)]
pub enum Instruction {
    Unreachable,
    Nop,
    Block(BlockType),
    Loop(BlockType),
    If(BlockType),
    Else,
    End,
    Br(u32),
    BrIf(u32),
    BrTable(BrTable),
    BrOnNull(u32),
    BrOnNonNull(u32),
    Return,
    Call(u32),
    CallRef(u32),
    CallIndirect(u32, u32, u32),
    ReturnCallRef(u32),
    ReturnCall(u32),
    ReturnCallIndirect(u32, u32),

    TryTable(TryTable),
    Throw(u32),
    ThrowRef,

    // Deprecated exception-handling instructions
    Try(BlockType),
    Delegate(u32),
    Catch(u32),
    CatchAll,
    Rethrow(u32),

    // Parametric instructions.
    Drop,
    Select,

    // Variable instructions.
    LocalGet(u32),
    LocalSet(u32),
    LocalTee(u32),
    GlobalGet(u32),
    GlobalSet(u32),

    // Memory instructions.
    I32Load(MemArg),
    I64Load(MemArg),
    F32Load(MemArg),
    F64Load(MemArg),
    I32Load8S(MemArg),
    I32Load8U(MemArg),
    I32Load16S(MemArg),
    I32Load16U(MemArg),
    I64Load8S(MemArg),
    I64Load8U(MemArg),
    I64Load16S(MemArg),
    I64Load16U(MemArg),
    I64Load32S(MemArg),
    I64Load32U(MemArg),
    I32Store(MemArg),
    I64Store(MemArg),
    F32Store(MemArg),
    F64Store(MemArg),
    I32Store8(MemArg),
    I32Store16(MemArg),
    I64Store8(MemArg),
    I64Store16(MemArg),
    I64Store32(MemArg),
    MemorySize(u32, u8),
    MemoryGrow(u32, u8),
    MemoryInit(u32, u32),
    DataDrop(u32),
    MemoryCopy(u32, u32),
    MemoryFill(u32),
    MemoryDiscard(u32),

    // Numeric instructions.
    I32Const(i32),
    I64Const(i64),
    F32Const(F32),
    F64Const(F64),
    I32Eqz,
    I32Eq,
    I32Ne,
    I32LtS,
    I32LtU,
    I32GtS,
    I32GtU,
    I32LeS,
    I32LeU,
    I32GeS,
    I32GeU,
    I64Eqz,
    I64Eq,
    I64Ne,
    I64LtS,
    I64LtU,
    I64GtS,
    I64GtU,
    I64LeS,
    I64LeU,
    I64GeS,
    I64GeU,
    F32Eq,
    F32Ne,
    F32Lt,
    F32Gt,
    F32Le,
    F32Ge,
    F64Eq,
    F64Ne,
    F64Lt,
    F64Gt,
    F64Le,
    F64Ge,
    I32Clz,
    I32Ctz,
    I32Popcnt,
    I32Add,
    I32Sub,
    I32Mul,
    I32DivS,
    I32DivU,
    I32RemS,
    I32RemU,
    I32And,
    I32Or,
    I32Xor,
    I32Shl,
    I32ShrS,
    I32ShrU,
    I32Rotl,
    I32Rotr,
    I64Clz,
    I64Ctz,
    I64Popcnt,
    I64Add,
    I64Sub,
    I64Mul,
    I64DivS,
    I64DivU,
    I64RemS,
    I64RemU,
    I64And,
    I64Or,
    I64Xor,
    I64Shl,
    I64ShrS,
    I64ShrU,
    I64Rotl,
    I64Rotr,
    F32Abs,
    F32Neg,
    F32Ceil,
    F32Floor,
    F32Trunc,
    F32Nearest,
    F32Sqrt,
    F32Add,
    F32Sub,
    F32Mul,
    F32Div,
    F32Min,
    F32Max,
    F32Copysign,
    F64Abs,
    F64Neg,
    F64Ceil,
    F64Floor,
    F64Trunc,
    F64Nearest,
    F64Sqrt,
    F64Add,
    F64Sub,
    F64Mul,
    F64Div,
    F64Min,
    F64Max,
    F64Copysign,
    I32WrapI64,
    I32TruncF32S,
    I32TruncF32U,
    I32TruncF64S,
    I32TruncF64U,
    I64ExtendI32S,
    I64ExtendI32U,
    I64TruncF32S,
    I64TruncF32U,
    I64TruncF64S,
    I64TruncF64U,
    F32ConvertI32S,
    F32ConvertI32U,
    F32ConvertI64S,
    F32ConvertI64U,
    F32DemoteF64,
    F64ConvertI32S,
    F64ConvertI32U,
    F64ConvertI64S,
    F64ConvertI64U,
    F64PromoteF32,
    I32ReinterpretF32,
    I64ReinterpretF64,
    F32ReinterpretI32,
    F64ReinterpretI64,
    I32Extend8S,
    I32Extend16S,
    I64Extend8S,
    I64Extend16S,
    I64Extend32S,
    I32TruncSatF32S,
    I32TruncSatF32U,
    I32TruncSatF64S,
    I32TruncSatF64U,
    I64TruncSatF32S,
    I64TruncSatF32U,
    I64TruncSatF64S,
    I64TruncSatF64U,

    // Reference types instructions.
    TypedSelect(ValType),
    RefNull(HeapType),
    RefIsNull,
    RefFunc(u32),
    RefEq,
    RefAsNonNull,

    // GC types instructions.
    StructNew(u32),
    StructNewDefault(u32),
    StructGet(u32, u32),
    StructGetS(u32, u32),
    StructGetU(u32, u32),
    StructSet(u32, u32),

    ArrayNew(u32),
    ArrayNewDefault(u32),
    ArrayNewFixed(u32, u32),
    ArrayNewData(u32, u32),
    ArrayNewElem(u32, u32),
    ArrayGet(u32),
    ArrayGetS(u32),
    ArrayGetU(u32),
    ArraySet(u32),
    ArrayLen,
    ArrayFill(u32),
    ArrayCopy(u32, u32),
    ArrayInitData(u32, u32),
    ArrayInitElem(u32, u32),
    RefTestNonNull(HeapType),
    RefTestNullable(HeapType),
    RefCastNonNull(HeapType),
    RefCastNullable(HeapType),
    BrOnCast(u32, RefType, RefType),
    BrOnCastFail(u32, RefType, RefType),
    AnyConvertExtern,
    ExternConvertAny,

    RefI31,
    I31GetS,
    I31GetU,

    // Bulk memory instructions.
    TableInit(u32, u32),
    ElemDrop(u32),
    TableFill(u32),
    TableSet(u32),
    TableGet(u32),
    TableGrow(u32),
    TableSize(u32),
    TableCopy(u32, u32),

    // SIMD instructions.
    V128Load(MemArg),
    V128Load8x8S(MemArg),
    V128Load8x8U(MemArg),
    V128Load16x4S(MemArg),
    V128Load16x4U(MemArg),
    V128Load32x2S(MemArg),
    V128Load32x2U(MemArg),
    V128Load8Splat(MemArg),
    V128Load16Splat(MemArg),
    V128Load32Splat(MemArg),
    V128Load64Splat(MemArg),
    V128Load32Zero(MemArg),
    V128Load64Zero(MemArg),
    V128Store(MemArg),
    V128Load8Lane(MemArg, Lane),
    V128Load16Lane(MemArg, Lane),
    V128Load32Lane(MemArg, Lane),
    V128Load64Lane(MemArg, Lane),
    V128Store8Lane(MemArg, Lane),
    V128Store16Lane(MemArg, Lane),
    V128Store32Lane(MemArg, Lane),
    V128Store64Lane(MemArg, Lane),
    V128Const(I128),
    I8x16Shuffle([Lane; 16]),
    I8x16ExtractLaneS(Lane),
    I8x16ExtractLaneU(Lane),
    I8x16ReplaceLane(Lane),
    I16x8ExtractLaneS(Lane),
    I16x8ExtractLaneU(Lane),
    I16x8ReplaceLane(Lane),
    I32x4ExtractLane(Lane),
    I32x4ReplaceLane(Lane),
    I64x2ExtractLane(Lane),
    I64x2ReplaceLane(Lane),
    F32x4ExtractLane(Lane),
    F32x4ReplaceLane(Lane),
    F64x2ExtractLane(Lane),
    F64x2ReplaceLane(Lane),
    I8x16Swizzle,
    I8x16Splat,
    I16x8Splat,
    I32x4Splat,
    I64x2Splat,
    F32x4Splat,
    F64x2Splat,
    I8x16Eq,
    I8x16Ne,
    I8x16LtS,
    I8x16LtU,
    I8x16GtS,
    I8x16GtU,
    I8x16LeS,
    I8x16LeU,
    I8x16GeS,
    I8x16GeU,
    I16x8Eq,
    I16x8Ne,
    I16x8LtS,
    I16x8LtU,
    I16x8GtS,
    I16x8GtU,
    I16x8LeS,
    I16x8LeU,
    I16x8GeS,
    I16x8GeU,
    I32x4Eq,
    I32x4Ne,
    I32x4LtS,
    I32x4LtU,
    I32x4GtS,
    I32x4GtU,
    I32x4LeS,
    I32x4LeU,
    I32x4GeS,
    I32x4GeU,
    I64x2Eq,
    I64x2Ne,
    I64x2LtS,
    I64x2GtS,
    I64x2LeS,
    I64x2GeS,
    F32x4Eq,
    F32x4Ne,
    F32x4Lt,
    F32x4Gt,
    F32x4Le,
    F32x4Ge,
    F64x2Eq,
    F64x2Ne,
    F64x2Lt,
    F64x2Gt,
    F64x2Le,
    F64x2Ge,
    V128Not,
    V128And,
    V128AndNot,
    V128Or,
    V128Xor,
    V128Bitselect,
    V128AnyTrue,
    I8x16Abs,
    I8x16Neg,
    I8x16Popcnt,
    I8x16AllTrue,
    I8x16Bitmask,
    I8x16NarrowI16x8S,
    I8x16NarrowI16x8U,
    I8x16Shl,
    I8x16ShrS,
    I8x16ShrU,
    I8x16Add,
    I8x16AddSatS,
    I8x16AddSatU,
    I8x16Sub,
    I8x16SubSatS,
    I8x16SubSatU,
    I8x16MinS,
    I8x16MinU,
    I8x16MaxS,
    I8x16MaxU,
    I8x16AvgrU,
    I16x8ExtAddPairwiseI8x16S,
    I16x8ExtAddPairwiseI8x16U,
    I16x8Abs,
    I16x8Neg,
    I16x8Q15MulrSatS,
    I16x8AllTrue,
    I16x8Bitmask,
    I16x8NarrowI32x4S,
    I16x8NarrowI32x4U,
    I16x8ExtendLowI8x16S,
    I16x8ExtendHighI8x16S,
    I16x8ExtendLowI8x16U,
    I16x8ExtendHighI8x16U,
    I16x8Shl,
    I16x8ShrS,
    I16x8ShrU,
    I16x8Add,
    I16x8AddSatS,
    I16x8AddSatU,
    I16x8Sub,
    I16x8SubSatS,
    I16x8SubSatU,
    I16x8Mul,
    I16x8MinS,
    I16x8MinU,
    I16x8MaxS,
    I16x8MaxU,
    I16x8AvgrU,
    I16x8ExtMulLowI8x16S,
    I16x8ExtMulHighI8x16S,
    I16x8ExtMulLowI8x16U,
    I16x8ExtMulHighI8x16U,
    I32x4ExtAddPairwiseI16x8S,
    I32x4ExtAddPairwiseI16x8U,
    I32x4Abs,
    I32x4Neg,
    I32x4AllTrue,
    I32x4Bitmask,
    I32x4ExtendLowI16x8S,
    I32x4ExtendHighI16x8S,
    I32x4ExtendLowI16x8U,
    I32x4ExtendHighI16x8U,
    I32x4Shl,
    I32x4ShrS,
    I32x4ShrU,
    I32x4Add,
    I32x4Sub,
    I32x4Mul,
    I32x4MinS,
    I32x4MinU,
    I32x4MaxS,
    I32x4MaxU,
    I32x4DotI16x8S,
    I32x4ExtMulLowI16x8S,
    I32x4ExtMulHighI16x8S,
    I32x4ExtMulLowI16x8U,
    I32x4ExtMulHighI16x8U,
    I64x2Abs,
    I64x2Neg,
    I64x2AllTrue,
    I64x2Bitmask,
    I64x2ExtendLowI32x4S,
    I64x2ExtendHighI32x4S,
    I64x2ExtendLowI32x4U,
    I64x2ExtendHighI32x4U,
    I64x2Shl,
    I64x2ShrS,
    I64x2ShrU,
    I64x2Add,
    I64x2Sub,
    I64x2Mul,
    I64x2ExtMulLowI32x4S,
    I64x2ExtMulHighI32x4S,
    I64x2ExtMulLowI32x4U,
    I64x2ExtMulHighI32x4U,
    F32x4Ceil,
    F32x4Floor,
    F32x4Trunc,
    F32x4Nearest,
    F32x4Abs,
    F32x4Neg,
    F32x4Sqrt,
    F32x4Add,
    F32x4Sub,
    F32x4Mul,
    F32x4Div,
    F32x4Min,
    F32x4Max,
    F32x4PMin,
    F32x4PMax,
    F64x2Ceil,
    F64x2Floor,
    F64x2Trunc,
    F64x2Nearest,
    F64x2Abs,
    F64x2Neg,
    F64x2Sqrt,
    F64x2Add,
    F64x2Sub,
    F64x2Mul,
    F64x2Div,
    F64x2Min,
    F64x2Max,
    F64x2PMin,
    F64x2PMax,
    I32x4TruncSatF32x4S,
    I32x4TruncSatF32x4U,
    F32x4ConvertI32x4S,
    F32x4ConvertI32x4U,
    I32x4TruncSatF64x2SZero,
    I32x4TruncSatF64x2UZero,
    F64x2ConvertLowI32x4S,
    F64x2ConvertLowI32x4U,
    F32x4DemoteF64x2Zero,
    F64x2PromoteLowF32x4,

    // Relaxed simd proposal
    I8x16RelaxedSwizzle,
    I32x4RelaxedTruncF32x4S,
    I32x4RelaxedTruncF32x4U,
    I32x4RelaxedTruncF64x2SZero,
    I32x4RelaxedTruncF64x2UZero,
    F32x4RelaxedMadd,
    F32x4RelaxedNmadd,
    F64x2RelaxedMadd,
    F64x2RelaxedNmadd,
    I8x16RelaxedLaneselect,
    I16x8RelaxedLaneselect,
    I32x4RelaxedLaneselect,
    I64x2RelaxedLaneselect,
    F32x4RelaxedMin,
    F32x4RelaxedMax,
    F64x2RelaxedMin,
    F64x2RelaxedMax,
    I16x8RelaxedQ15mulrS,
    I16x8RelaxedDotI8x16I7x16S,
    I32x4RelaxedDotI8x16I7x16AddS,

    // Atomic instructions (the threads proposal)
    MemoryAtomicNotify(MemArg),
    MemoryAtomicWait32(MemArg),
    MemoryAtomicWait64(MemArg),
    AtomicFence,
    I32AtomicLoad(MemArg),
    I64AtomicLoad(MemArg),
    I32AtomicLoad8U(MemArg),
    I32AtomicLoad16U(MemArg),
    I64AtomicLoad8U(MemArg),
    I64AtomicLoad16U(MemArg),
    I64AtomicLoad32U(MemArg),
    I32AtomicStore(MemArg),
    I64AtomicStore(MemArg),
    I32AtomicStore8(MemArg),
    I32AtomicStore16(MemArg),
    I64AtomicStore8(MemArg),
    I64AtomicStore16(MemArg),
    I64AtomicStore32(MemArg),
    I32AtomicRmwAdd(MemArg),
    I64AtomicRmwAdd(MemArg),
    I32AtomicRmw8AddU(MemArg),
    I32AtomicRmw16AddU(MemArg),
    I64AtomicRmw8AddU(MemArg),
    I64AtomicRmw16AddU(MemArg),
    I64AtomicRmw32AddU(MemArg),
    I32AtomicRmwSub(MemArg),
    I64AtomicRmwSub(MemArg),
    I32AtomicRmw8SubU(MemArg),
    I32AtomicRmw16SubU(MemArg),
    I64AtomicRmw8SubU(MemArg),
    I64AtomicRmw16SubU(MemArg),
    I64AtomicRmw32SubU(MemArg),
    I32AtomicRmwAnd(MemArg),
    I64AtomicRmwAnd(MemArg),
    I32AtomicRmw8AndU(MemArg),
    I32AtomicRmw16AndU(MemArg),
    I64AtomicRmw8AndU(MemArg),
    I64AtomicRmw16AndU(MemArg),
    I64AtomicRmw32AndU(MemArg),
    I32AtomicRmwOr(MemArg),
    I64AtomicRmwOr(MemArg),
    I32AtomicRmw8OrU(MemArg),
    I32AtomicRmw16OrU(MemArg),
    I64AtomicRmw8OrU(MemArg),
    I64AtomicRmw16OrU(MemArg),
    I64AtomicRmw32OrU(MemArg),
    I32AtomicRmwXor(MemArg),
    I64AtomicRmwXor(MemArg),
    I32AtomicRmw8XorU(MemArg),
    I32AtomicRmw16XorU(MemArg),
    I64AtomicRmw8XorU(MemArg),
    I64AtomicRmw16XorU(MemArg),
    I64AtomicRmw32XorU(MemArg),
    I32AtomicRmwXchg(MemArg),
    I64AtomicRmwXchg(MemArg),
    I32AtomicRmw8XchgU(MemArg),
    I32AtomicRmw16XchgU(MemArg),
    I64AtomicRmw8XchgU(MemArg),
    I64AtomicRmw16XchgU(MemArg),
    I64AtomicRmw32XchgU(MemArg),
    I32AtomicRmwCmpxchg(MemArg),
    I64AtomicRmwCmpxchg(MemArg),
    I32AtomicRmw8CmpxchgU(MemArg),
    I32AtomicRmw16CmpxchgU(MemArg),
    I64AtomicRmw8CmpxchgU(MemArg),
    I64AtomicRmw16CmpxchgU(MemArg),
    I64AtomicRmw32CmpxchgU(MemArg),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Expr(pub Vec<Instruction>);

#[derive(Clone, Debug, PartialEq)]
pub struct ConstExpr(pub Vec<Instruction>);

impl Instruction {
    pub fn is_const(&self) -> bool {
        matches!(
            self,
            Instruction::F64Const(_)
                | Instruction::F32Const(_)
                | Instruction::I32Const(_)
                | Instruction::I64Const(_)
                | Instruction::V128Const(_)
                | Instruction::GlobalGet(_)
                | Instruction::RefFunc(_)
                | Instruction::RefNull(_)
                | Instruction::RefIsNull
        )
    }
}

macro_rules! define_visit_operator {
    ($( @$proposal:ident $op:ident $({ $($arg:ident: $argty:ty),* })? => $visit:ident)*) => {
        $(
            fn $visit(&mut self $($(,$arg: $argty)*)?) -> Self::Output {
                Instruction::$op $(( $($arg.into()),* ))?
            }
        )*
    };
}

pub struct InstructionVisitor;

impl<'a> VisitOperator<'a> for InstructionVisitor {
    type Output = Instruction;

    wasmparser::for_each_operator!(define_visit_operator);
}

impl<'a, 'b> TryFrom<&'b mut Decoder<'a>> for Instruction {
    type Error = Error;

    fn try_from(decoder: &'b mut Decoder<'a>) -> Result<Self, Self::Error> {
        let mut reader = BinaryReader::new_with_offset(
            decoder.remaining_slice(),
            decoder.reader.position() as usize,
        );
        let instr = reader.visit_operator(&mut InstructionVisitor)?;
        decoder
            .reader
            .set_position(reader.original_position() as u64);
        Ok(instr)
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct MemArg {
    align: u8,
    offset: u64,
}

impl From<wasmparser::MemArg> for MemArg {
    fn from(value: wasmparser::MemArg) -> Self {
        MemArg {
            align: value.align,
            offset: value.offset,
        }
    }
}

pub type Lane = u8;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Catch {
    Catch { tag: u32, label: u32 },
    CatchRef { tag: u32, label: u32 },
    CatchAll { label: u32 },
    CatchAllRef { label: u32 },
}

impl From<wasmparser::Catch> for Catch {
    fn from(value: wasmparser::Catch) -> Self {
        match value {
            wasmparser::Catch::One { tag, label } => Catch::Catch { tag, label },
            wasmparser::Catch::OneRef { tag, label } => Catch::CatchRef { tag, label },
            wasmparser::Catch::All { label } => Catch::CatchAll { label },
            wasmparser::Catch::AllRef { label } => Catch::CatchAllRef { label },
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct TryTable {
    pub ty: BlockType,
    pub catches: Vec<Catch>,
}

impl From<wasmparser::TryTable> for TryTable {
    fn from(value: wasmparser::TryTable) -> Self {
        TryTable {
            ty: value.ty.into(),
            catches: value.catches.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct BrTable {
    targets: crate::SVec<u32>,
    default: u32,
}

impl<'a> From<wasmparser::BrTable<'a>> for BrTable {
    fn from(value: wasmparser::BrTable<'a>) -> Self {
        let targets = value.targets().collect::<Result<crate::SVec<_>, _>>().unwrap();
        BrTable {
            targets,
            default: value.default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct F32(pub f32);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct F64(pub f64);

impl From<wasmparser::Ieee32> for F32 {
    fn from(value: wasmparser::Ieee32) -> Self {
        F32(f32::from_le_bytes(value.bits().to_le_bytes()))
    }
}

impl From<wasmparser::Ieee64> for F64 {
    fn from(value: wasmparser::Ieee64) -> Self {
        F64(f64::from_le_bytes(value.bits().to_le_bytes()))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct I128(pub i128);

impl From<wasmparser::V128> for I128 {
    fn from(value: wasmparser::V128) -> Self {
        I128(value.i128())
    }
}

impl From<wasmparser::HeapType> for HeapType {
    fn from(value: wasmparser::HeapType) -> Self {
        match value {
            wasmparser::HeapType::Concrete(ty) => HeapType::Concrete(ty.as_module_index().unwrap()),
            wasmparser::HeapType::Func => HeapType::Func,
            wasmparser::HeapType::Extern => HeapType::Extern,
            wasmparser::HeapType::Any => HeapType::Any,
            wasmparser::HeapType::None => HeapType::None,
            wasmparser::HeapType::NoExtern => HeapType::NoExtern,
            wasmparser::HeapType::NoFunc => HeapType::NoFunc,
            wasmparser::HeapType::Eq => HeapType::Eq,
            wasmparser::HeapType::Struct => HeapType::Struct,
            wasmparser::HeapType::Array => HeapType::Array,
            wasmparser::HeapType::I31 => HeapType::I31,
            wasmparser::HeapType::Exn => HeapType::Exn,
            wasmparser::HeapType::NoExn => HeapType::NoExn,
        }
    }
}
