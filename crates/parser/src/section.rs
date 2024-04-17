use smol_str::SmolStr;

use crate::error::Error;
use crate::instruction::{ConstExpr, Expr};
use crate::types::{FuncType, Global, GlobalType, MemoryType, RefType, TableType, ValType};


#[derive(Debug, PartialEq, Eq)]
pub enum SectionId {
    Custom = 0x00,
    Type = 0x01,
    Import = 0x02,
    Function = 0x03,
    Table = 0x04,
    Memory = 0x05,
    Global = 0x06,
    Export = 0x07,
    Start = 0x08,
    Element = 0x09,
    Code = 0x0a,
    Data = 0x0b,
    DataCount = 0x0c,
}

impl TryFrom<u8> for SectionId {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(SectionId::Custom),
            0x01 => Ok(SectionId::Type),
            0x02 => Ok(SectionId::Import),
            0x03 => Ok(SectionId::Function),
            0x04 => Ok(SectionId::Table),
            0x05 => Ok(SectionId::Memory),
            0x06 => Ok(SectionId::Global),
            0x07 => Ok(SectionId::Export),
            0x08 => Ok(SectionId::Start),
            0x09 => Ok(SectionId::Element),
            0x0a => Ok(SectionId::Code),
            0x0b => Ok(SectionId::Data),
            0x0c => Ok(SectionId::DataCount),
            _ => Err(Error::InvalidSectionId(value)),
        }
    }
}

#[derive(Debug, Default)]
pub struct CustomSection {
    pub name: SmolStr,
    pub data: Vec<u8>,
}

#[derive(Debug, Default)]
pub struct TypeSection(pub Vec<TypeSectionTy>);

#[derive(Debug, Default)]
pub struct ImportSection(pub Vec<Import>);

#[derive(Debug, Default)]
pub struct FunctionSection(pub crate::SVec<u32>);

#[derive(Debug, Default)]
pub struct TableSection(pub crate::SVec<TableType>);

#[derive(Debug, Default)]
pub struct MemorySection(pub crate::SVec<MemoryType>);

#[derive(Debug, Default)]
pub struct GlobalSection(pub Vec<Global>);

#[derive(Debug, Default)]
pub struct ExportSection(pub Vec<Export>);

#[derive(Debug, Default)]
pub struct StartSection(pub u32);

#[derive(Debug, Default)]
pub struct ElementSection(pub Vec<Element>);

#[derive(Debug, Default)]
pub struct CodeSection(pub Vec<Code>);

#[derive(Debug, Default)]
pub struct DataSection(pub Vec<Data>);

#[derive(Debug, Default)]
pub struct DataCountSection(pub Option<u32>);

#[derive(Debug)]
pub enum TypeSectionTy {
    Func(FuncType),
}

#[derive(Debug)]
pub struct Import {
    pub module_name: SmolStr,
    pub field_name: SmolStr,
    pub kind: ImportKind,
}

#[derive(Debug)]
pub enum ImportKind {
    Func(u32),
    Table(TableType),
    Memory(MemoryType),
    Global(GlobalType),
}

#[derive(Debug)]
pub struct Export {
    pub name: SmolStr,
    pub kind: ExportKind,
}

#[derive(Debug)]
pub enum ExportKind {
    Func(u32),
    Table(u32),
    Mem(u32),
    Global(u32),
}

#[derive(Debug)]
pub struct Element {
    pub ty: RefType,
    pub init: crate::SVec<ConstExpr>,
    pub kind: ElementKind,
}

#[derive(Debug)]
pub enum ElementKind {
    Passive,
    Active {
        table: Option<u32>,
        offset: ConstExpr,
    },
    Declared,
}

#[derive(Debug)]
pub struct Code {
    pub size: u32,
    pub locals: crate::SVec<Locals>,
    pub expr: Expr,
}

#[derive(Debug)]
pub struct Locals {
    pub n: u32,
    pub ty: ValType,
}

#[derive(Debug)]
pub struct Data {
    pub init: Vec<u8>,
    pub kind: DataKind,
}

#[derive(Debug)]
pub enum DataKind {
    Passive,
    Active {
        memory: u32,
        offset: ConstExpr,
    },
}
