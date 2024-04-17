use std::io::{Cursor, Read};

use smol_str::SmolStr;

use crate::error::Error;
use crate::instruction::{ConstExpr, Expr, Instruction};
use crate::types::{
    BlockType, GlobalType, HeapType, Limit, MemoryType, RefType, ResultType, TableType, ValType,
};

#[derive(Clone)]
pub struct Decoder<'a> {
    pub reader: Cursor<&'a [u8]>,
}

impl<'a> Decoder<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Decoder {
            reader: Cursor::new(bytes),
        }
    }

    pub fn is_empty(&mut self) -> bool {
        self.reader.get_ref().len() as u64 - self.reader.position() <= 1
    }

    /// copy from [Cursor::remaining_slice], because it is unstable
    pub fn remaining_slice(&self) -> &[u8] {
        let len = self
            .reader
            .position()
            .min(self.reader.get_ref().len() as u64);
        &self.reader.get_ref()[(len as usize)..]
    }

    pub fn peek(&self) -> u8 {
        self.reader.get_ref()[self.reader.position() as usize]
    }

    pub fn read_n<const N: usize>(&mut self) -> Result<[u8; N], Error> {
        let mut buf = [0u8; N];
        self.reader
            .read_exact(&mut buf)
            .map(|_| buf)
            .map_err(Into::into)
    }

    pub fn read_bytes(&mut self, len: usize) -> Result<Vec<u8>, Error> {
        let mut buf = vec![0u8; len];
        self.reader
            .read_exact(&mut buf)
            .map(|_| buf)
            .map_err(Into::into)
    }

    pub fn read_str(&mut self) -> Result<SmolStr, Error> {
        let len = self.read_var_u32()? as usize;
        let end = self.reader.position() as usize + len;
        let buf = &self.remaining_slice()[..len];
        let s = SmolStr::new(std::str::from_utf8(&buf)?);
        self.reader.set_position(end as u64);
        Ok(s)
    }

    pub fn read_u8(&mut self) -> Result<u8, Error> {
        self.read_n::<1>().map(|b| b[0])
    }

    pub fn read_var_i32(&mut self) -> Result<i32, Error> {
        leb128::read::signed(&mut self.reader)
            .map(|i| i as i32)
            .map_err(Into::into)
    }

    pub fn read_u32(&mut self) -> Result<u32, Error> {
        self.read_n::<4>().map(u32::from_le_bytes)
    }

    pub fn read_var_u32(&mut self) -> Result<u32, Error> {
        leb128::read::unsigned(&mut self.reader)
            .map(|i| i as u32)
            .map_err(Into::into)
    }

    pub fn read_var_i64(&mut self) -> Result<i64, Error> {
        leb128::read::signed(&mut self.reader).map_err(Into::into)
    }

    pub fn read_f32(&mut self) -> Result<f32, Error> {
        self.read_n::<4>().map(f32::from_le_bytes)
    }

    pub fn read_f64(&mut self) -> Result<f64, Error> {
        self.read_n::<8>().map(f64::from_le_bytes)
    }

    /// copy from https://github.com/bytecodealliance/wasm-tools/blob/main/crates/wasmparser/src/binary_reader.rs#L557
    pub fn read_var_s33(&mut self) -> Result<i64, Error> {
        let byte = self.read_u8()?;
        if (byte & 0x80) == 0 {
            return Ok(((byte as i8) << 1) as i64 >> 1);
        }

        let mut result = (byte & 0x7F) as i64;
        let mut shift = 7;
        loop {
            let byte = self.read_u8()?;
            result |= ((byte & 0x7F) as i64) << shift;
            if shift >= 25 {
                let continuation_bit = (byte & 0x80) != 0;
                let sign_and_unused_bit = (byte << 1) as i8 >> (33 - shift);
                if continuation_bit || (sign_and_unused_bit != 0 && sign_and_unused_bit != -1) {
                    return Err(Error::Decode(std::io::Error::other(
                        "invalid var_s33: integer representation too long",
                    )));
                }
                return Ok(result);
            }
            shift += 7;
            if (byte & 0x80) == 0 {
                break;
            }
        }
        let ashift = 64 - shift;
        Ok((result << ashift) >> ashift)
    }

    pub fn read_svec<F, T>(&mut self, ele: F) -> Result<crate::SVec<T>, Error>
    where
        F: Fn(&mut Self) -> Result<T, Error>,
    {
        let len = self.read_var_u32()?;
        (0..len)
            .map(|_| ele(self))
            .collect::<Result<crate::SVec<_>, _>>()
    }

    
    pub fn read_vec<F, T>(&mut self, ele: F) -> Result<Vec<T>, Error>
    where
        F: Fn(&mut Self) -> Result<T, Error>,
    {
        let len = self.read_var_u32()?;
        (0..len)
            .map(|_| ele(self))
            .collect::<Result<Vec<_>, _>>()
    }

    pub fn slice_with(&mut self, len: u64) -> Self {
        let pos = self.reader.position();
        let end = pos + len;
        self.reader.set_position(end);
        Decoder {
            reader: Cursor::new(&self.reader.get_ref()[pos as usize..end as usize]),
        }
    }

    // wasm type

    pub fn read_valtype(&mut self) -> Result<ValType, Error> {
        ValType::try_from(self.read_u8()?)
    }

    pub fn read_reftype(&mut self) -> Result<RefType, Error> {
        RefType::try_from(self.read_u8()?)
    }

    pub fn read_resulttype(&mut self) -> Result<ResultType, Error> {
        let len = self.read_var_u32()?;

        Ok((0..len)
            .map(|_| self.read_valtype())
            .collect::<Result<crate::SVec<_>, _>>()
            .map(ResultType)?)
    }

    pub fn read_limit(&mut self) -> Result<Limit, Error> {
        let limited = self.read_u8()? == 0x01;
        let min = self.read_var_u32()?;
        Ok(Limit {
            min,
            max: limited.then(|| self.read_var_u32()).transpose()?,
        })
    }

    pub fn read_memtype(&mut self) -> Result<MemoryType, Error> {
        self.read_limit().map(MemoryType)
    }

    pub fn read_tabletype(&mut self) -> Result<TableType, Error> {
        let element = self.read_reftype()?;
        let limit = self.read_limit()?;
        Ok(TableType { element, limit })
    }

    pub fn read_globaltype(&mut self) -> Result<GlobalType, Error> {
        let valtype = self.read_valtype()?;
        let mutable = self.read_u8()? == 0x01;
        Ok(GlobalType {
            ty: valtype,
            mutable,
        })
    }

    pub fn read_const_expr(&mut self) -> Result<ConstExpr, Error> {
        let mut instrs = Vec::new();
        loop {
            let instr = self.read_instruction()?;
            let is_end = instr == Instruction::End;

            if !instr.is_const() && !is_end {
                return Err(Error::InvalidConstExprOpcode(instr));
            }

            instrs.push(instr);

            if is_end {
                break;
            }
        }
        Ok(ConstExpr(instrs))
    }

    pub fn read_expr(&mut self) -> Result<Expr, Error> {
        let mut block_depth = 0;
        let mut instrs = Vec::new();
        loop {
            let instr = self.read_instruction()?;
            let is_end = instr == Instruction::End;
            let start_block = matches!(
                &instr,
                Instruction::Block(_)
                    | Instruction::Loop(_)
                    | Instruction::If(_)
                    | Instruction::Try(_)
                    | Instruction::TryTable(_)
            );

            instrs.push(instr);

            if start_block {
                block_depth += 1;
            }

            if is_end {
                if block_depth == 0 {
                    break;
                }
                block_depth -= 1;
            }
        }
        Ok(Expr(instrs))
    }

    pub fn read_instruction(&mut self) -> Result<Instruction, Error> {
        Instruction::try_from(self)
    }

    pub fn read_block_type(&mut self) -> Result<BlockType, Error> {
        let peek = self.peek();

        Ok(if peek == 0x40 {
            self.read_u8()?;
            BlockType::Empty
        } else if let Ok(valtype) = ValType::try_from(peek) {
            self.read_u8()?;
            BlockType::Type(valtype)
        } else {
            let idx = self.read_var_s33()?;
            BlockType::FuncType(idx.try_into().map_err(|_| Error::InvalidBlockType)?)
        })
    }

    pub fn read_heaptype(&mut self) -> Result<HeapType, Error> {
        HeapType::try_from(self.read_var_s33()?)
    }
}
