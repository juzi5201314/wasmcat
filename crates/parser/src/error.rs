use thiserror::Error;

use crate::instruction::Instruction;

#[derive(Error, Debug)]
pub enum Error {
    #[error("decoder io error: {0}")]
    Decode(#[from] std::io::Error),
    #[error("decode leb128 error: {0}")]
    Leb128(#[from] leb128::read::Error),
    #[error("decode utf8 error: {0}")]
    Utf8(#[from] std::str::Utf8Error),

    #[error("invalid wasm format")]
    InvalidWasm,
    #[error("invalid block type")]
    InvalidBlockType,
    #[error("invalid section id: {0:#04x}")]
    InvalidSectionId(u8),
    #[error("invalid type: expect `{0}`, but found {1:#04x}")]
    InvalidType(&'static str, u8),
    #[error("invalid type tag: {0:#04x}")]
    InvalidTypeTag(u8),
    #[error("invalid import kind: {0:#04x}")]
    InvalidImportKind(u8),
    #[error("invalid export kind: {0:#04x}")]
    InvalidExportKind(u8),
    #[error("invalid instruction: {0:#04x}")]
    InvalidInstruction(u8),
    #[error("invalid expr end code: {0:#04x}")]
    InvalidExprEnd(u8),
    #[error("invalid flags byte in {1}: {0}")]
    InvalidFlags(u32, &'static str),
    #[error("invalid const expr instruction: {0:?}, it not const")]
    InvalidConstExprOpcode(Instruction),
    #[error("invalid elemtype {0:#04x} of table, must be funcref")]
    InvalidElmType(u8),
    #[error("unexpected 0xfc prefix: {0}")]
    Unexpected0xfc(u32),

    #[error("section {0:#04x} should be {1} bytes but the parser consumes {2}")]
    SectionOutOfBounds(u8, u32, u32),

    #[error("BinaryReaderError: {0}")]
    BinaryReaderErr(#[from] wasmparser::BinaryReaderError),

    #[error("{0}")]
    Other(&'static str),
}