use crate::error::Error;
use crate::instruction::ConstExpr;

macro_rules! ty_enum {
    ($ty:ident { $($ele:tt = $val:tt,)* }) => {
        #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
        pub enum $ty {
            $($ele = $val,)*
        }

        impl TryFrom<u8> for $ty {
            type Error = Error;

            fn try_from(value: u8) -> Result<Self, Self::Error> {
                match value {
                    $($val => Ok($ty::$ele),)*
                    _ => Err(Error::InvalidType(stringify!($ty), value)),
                }
            }
        }
    };
}

ty_enum!(RefType {
    FuncRef = 0x70,
    ExternRef = 0x6f,
});

impl From<wasmparser::RefType> for RefType {
    fn from(value: wasmparser::RefType) -> Self {
        if value.is_func_ref() {
            RefType::FuncRef
        } else if value.is_extern_ref() {
            RefType::ExternRef
        } else {
            panic!("wasm validation error")
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum ValType {
    I32,
    I64,
    F32,
    F64,
    V128,
    Ref(RefType),
}

impl TryFrom<u8> for ValType {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0x7f => ValType::I32,
            0x7e => ValType::I64,
            0x7d => ValType::F32,
            0x7c => ValType::F64,
            0x7b => ValType::V128,
            0x70 => ValType::Ref(RefType::FuncRef),
            0x6f => ValType::Ref(RefType::ExternRef),
            _ => return Err(Error::InvalidType("ValType", value)),
        })
    }
}

impl From<wasmparser::ValType> for ValType {
    fn from(value: wasmparser::ValType) -> Self {
        match value {
            wasmparser::ValType::I32 => ValType::I32,
            wasmparser::ValType::I64 => ValType::I64,
            wasmparser::ValType::F32 => ValType::F32,
            wasmparser::ValType::F64 => ValType::F64,
            wasmparser::ValType::V128 => ValType::V128,
            wasmparser::ValType::Ref(rt) => ValType::Ref(rt.into()),
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum ExternType {
    Func,
    Table,
    Mem,
    Global,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ResultType(pub crate::SVec<ValType>);

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct Limit {
    pub min: u32,
    pub max: Option<u32>,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct TableType {
    pub element: RefType,
    pub limit: Limit,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct GlobalType {
    pub ty: ValType,
    pub mutable: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Global {
    pub ty: GlobalType,
    pub expr: ConstExpr,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct MemoryType(pub Limit);

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct FuncType {
    pub params: ResultType,
    pub results: ResultType,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum BlockType {
    Empty,
    Type(ValType),
    FuncType(u32),
}

impl From<wasmparser::BlockType> for BlockType {
    fn from(value: wasmparser::BlockType) -> Self {
        match value {
            wasmparser::BlockType::Empty => BlockType::Empty,
            wasmparser::BlockType::Type(v) => BlockType::Type(v.into()),
            wasmparser::BlockType::FuncType(idx) => BlockType::FuncType(idx),
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum HeapType {
    Func,
    Extern,

    // [gc proposal]: https://github.com/WebAssembly/gc
    /// The common supertype (a.k.a. top) of all internal types.
    Any,

    /// The common subtype (a.k.a. bottom) of all internal types.
    None,

    /// The common subtype (a.k.a. bottom) of all external types.
    NoExtern,

    /// The common subtype (a.k.a. bottom) of all function types.
    NoFunc,

    /// the common supertype of all referenceable types on which comparison
    /// (ref.eq) is allowed (this may include host-defined external types).
    Eq,

    /// The common supertype of all struct types.
    Struct,

    /// The common supertype of all array types.
    Array,

    /// The type of unboxed scalars
    I31,

    /// The abstract `exception` heap type.
    Exn,

    /// The abstract `noexn` heap type.
    NoExn,

    Concrete(u32),
}

impl TryFrom<i64> for HeapType {
    type Error = Error;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        if value >= 0 {
            Ok(HeapType::Concrete(value as u32))
        } else {
            // https://github.com/WebAssembly/gc/blob/main/proposals/gc/MVP.md#heap-types-2
            match value as i8 {
                -0x0d => Ok(HeapType::NoFunc),
                -0x0e => Ok(HeapType::NoExtern),
                -0x0f => Ok(HeapType::None),
                -0x10 => Ok(HeapType::Func),
                -0x11 => Ok(HeapType::Exn),
                -0x12 => Ok(HeapType::Any),
                -0x13 => Ok(HeapType::Eq),
                -0x14 => Ok(HeapType::I31),
                -0x15 => Ok(HeapType::Struct),
                -0x16 => Ok(HeapType::Array),
                op => Err(Error::InvalidType("HeapType", (op + 0x7f) as u8 + 0x01)),
            }
        }
    }
}
