use crate::decode::Decoder;
use crate::error::Error;
use crate::instruction::{ConstExpr, Instruction};
use crate::module::Module;
use crate::section::{
    Code, CodeSection, CustomSection, Data, DataCountSection, DataKind, DataSection, Element,
    ElementKind, ElementSection, Export, ExportKind, ExportSection, FunctionSection, GlobalSection,
    Import, ImportKind, ImportSection, Locals, MemorySection, SectionId, StartSection,
    TableSection, TypeSection, TypeSectionTy,
};
use crate::types::{FuncType, Global, RefType};

pub struct ModuleParser<'a> {
    pub decoder: Decoder<'a>,
}

impl<'a> ModuleParser<'a> {
    fn parse_header(&mut self) -> Result<u32, Error> {
        let magic = self.decoder.read_n::<4>()?;
        let version = self.decoder.read_u32()?;
        if &magic != b"\0asm" {
            Err(Error::InvalidWasm)
        } else {
            Ok(version)
        }
    }

    pub fn read_section_header(&mut self) -> Result<(SectionId, u32), Error> {
        let id = self.decoder.read_u8()?;
        let size = self.decoder.read_var_u32()?;
        Ok((SectionId::try_from(id)?, size))
    }

    /// Parse modules and code sections in parallel, 
    /// this should speed up (2-5x) the parsing of large wasm files(large code sections), 
    /// based on the benchmark results, it is recommended to only use this method on wasm files >100kb
    ///
    /// ```
    /// test big_1900kb_wasm               ... bench:   2,928,385 ns/iter (+/- 664,241) = 677 MB/s
    /// test big_1900kb_wasm_parallel      ... bench:     640,027 ns/iter (+/- 73,691) = 3099 MB/s
    /// test medium_180kb_wasm             ... bench:     903,667 ns/iter (+/- 15,438) = 207 MB/s
    /// test medium_180kb_wasm_parallel    ... bench:     380,636 ns/iter (+/- 17,981) = 493 MB/s
    /// test small_30kb_wasm               ... bench:     558,580 ns/iter (+/- 77,136) = 53 MB/s
    /// test small_30kb_wasm_parallel      ... bench:     561,020 ns/iter (+/- 9,287) = 53 MB/s
    /// test very_big_5270kb_wasm          ... bench:  99,321,540 ns/iter (+/- 7,110,395) = 54 MB/s
    /// test very_big_5270kb_wasm_parallel ... bench:  37,908,100 ns/iter (+/- 8,038,568) = 142 MB/s
    /// ```
    #[cfg_attr(docsrs, doc(cfg(feature = "parallel")))]
    #[cfg(feature = "parallel")]
    pub fn par_parse(mut self) -> Result<Module, Error> {
        let mut module = Module::default();
        let mut sections = Vec::new();

        module.version = self.parse_header()?;

        while !self.decoder.is_empty() {
            let (id, size) = self.read_section_header()?;
            let start = self.decoder.reader.position();
            let end = self.decoder.reader.position() + size as u64;
            sections.push((id, start as usize, end as usize));
            self.decoder.reader.set_position(end);
        }

        use rayon::iter::IntoParallelIterator;
        use rayon::iter::ParallelIterator;
        use std::sync::Arc;
        let packed = Arc::new(spin::Mutex::new(module));
        sections.into_par_iter().try_for_each(|(id, start, end)| {
            let data = &self.decoder.reader.get_ref()[start..end];
            let mut decoder = Decoder::new(&data);
            match id {
                SectionId::Custom => {
                    let name = decoder.read_str()?;
                    let data =
                        decoder.read_bytes(data.len() - decoder.reader.position() as usize)?;
                    packed
                        .lock()
                        .custom_sections
                        .push(CustomSection { name, data });
                }
                SectionId::Type => {
                    packed.lock().type_section = Self::parse_type_section(&mut decoder)?;
                }
                SectionId::Import => {
                    packed.lock().import_section = Self::parse_import_section(&mut decoder)?;
                }
                SectionId::Function => {
                    packed.lock().func_section = Self::parse_function_section(&mut decoder)?;
                }
                SectionId::Table => {
                    packed.lock().table_section = Self::parse_table_section(&mut decoder)?;
                }
                SectionId::Memory => {
                    packed.lock().memory_section = Self::parse_memory_section(&mut decoder)?;
                }
                SectionId::Global => {
                    packed.lock().global_section = Self::parse_global_section(&mut decoder)?;
                }
                SectionId::Export => {
                    packed.lock().export_section = Self::parse_export_section(&mut decoder)?;
                }
                SectionId::Start => {
                    packed.lock().start_section = Self::parse_start_section(&mut decoder)?;
                }
                SectionId::Element => {
                    packed.lock().element_section = Self::parse_element_section(&mut decoder)?;
                }
                SectionId::Code => {
                    packed.lock().code_section = Self::par_parse_code_section(&mut decoder)?;
                }
                SectionId::Data => {
                    packed.lock().data_section = Self::parse_data_section(&mut decoder)?;
                }
                SectionId::DataCount => {
                    packed.lock().data_count_section =
                        Self::parse_data_count_section(&mut decoder)?;
                }
            }
            Result::<(), Error>::Ok(())
        })?;

        Ok(spin::Mutex::into_inner(Arc::into_inner(packed).unwrap()))
    }

    pub fn parse(mut self) -> Result<Module, Error> {
        let mut module = Module::default();

        module.version = self.parse_header()?;

        while !self.decoder.is_empty() {
            let (id, size) = self.read_section_header()?;
            let end = self.decoder.reader.position() + size as u64;

            match id {
                SectionId::Custom => {
                    let name = self.decoder.read_str()?;
                    let data = self
                        .decoder
                        .read_bytes((end - self.decoder.reader.position()) as usize)?;
                    module.custom_sections.push(CustomSection { name, data });
                }
                SectionId::Type => {
                    module.type_section = Self::parse_type_section(&mut self.decoder)?;
                }
                SectionId::Import => {
                    module.import_section = Self::parse_import_section(&mut self.decoder)?;
                }
                SectionId::Function => {
                    module.func_section = Self::parse_function_section(&mut self.decoder)?;
                }
                SectionId::Table => {
                    module.table_section = Self::parse_table_section(&mut self.decoder)?;
                }
                SectionId::Memory => {
                    module.memory_section = Self::parse_memory_section(&mut self.decoder)?;
                }
                SectionId::Global => {
                    module.global_section = Self::parse_global_section(&mut self.decoder)?;
                }
                SectionId::Export => {
                    module.export_section = Self::parse_export_section(&mut self.decoder)?;
                }
                SectionId::Start => {
                    module.start_section = Self::parse_start_section(&mut self.decoder)?;
                }
                SectionId::Element => {
                    module.element_section = Self::parse_element_section(&mut self.decoder)?;
                }
                SectionId::Code => {
                    module.code_section = Self::parse_code_section(&mut self.decoder)?;
                }
                SectionId::Data => {
                    module.data_section = Self::parse_data_section(&mut self.decoder)?;
                }
                SectionId::DataCount => {
                    module.data_count_section = Self::parse_data_count_section(&mut self.decoder)?;
                }
            }

            if self.decoder.reader.position() != end {
                return Err(Error::SectionOutOfBounds(
                    id as u8,
                    size,
                    (self.decoder.reader.position() - (end - size as u64)) as u32,
                ));
            }
        }

        Ok(module)
    }

    fn parse_type_section(decoder: &mut Decoder) -> Result<TypeSection, Error> {
        decoder
            .read_vec(|decoder| {
                let type_tag = decoder.read_u8()?;
                match type_tag {
                    0x60 => {
                        let params = decoder.read_resulttype()?;
                        let results = decoder.read_resulttype()?;

                        let func_type = FuncType { params, results };
                        Ok(TypeSectionTy::Func(func_type))
                    }
                    _ => Err(Error::InvalidTypeTag(type_tag)),
                }
            })
            .map(TypeSection)
    }

    fn parse_import_section(decoder: &mut Decoder) -> Result<ImportSection, Error> {
        decoder
            .read_vec(|decoder| {
                let module_name = decoder.read_str()?;
                let field_name = decoder.read_str()?;
                let kind = decoder.read_u8()?;
                let kind = match kind {
                    0x00 => ImportKind::Func(decoder.read_var_u32()?),
                    0x01 => ImportKind::Table(decoder.read_tabletype()?),
                    0x02 => ImportKind::Memory(decoder.read_memtype()?),
                    0x03 => ImportKind::Global(decoder.read_globaltype()?),
                    _ => return Err(Error::InvalidImportKind(kind)),
                };

                Ok(Import {
                    module_name,
                    field_name,
                    kind,
                })
            })
            .map(ImportSection)
    }

    fn parse_function_section(decoder: &mut Decoder) -> Result<FunctionSection, Error> {
        decoder
            .read_svec(Decoder::read_var_u32)
            .map(FunctionSection)
    }

    fn parse_table_section(decoder: &mut Decoder) -> Result<TableSection, Error> {
        decoder.read_svec(Decoder::read_tabletype).map(TableSection)
    }

    fn parse_memory_section(decoder: &mut Decoder) -> Result<MemorySection, Error> {
        decoder.read_svec(Decoder::read_memtype).map(MemorySection)
    }

    fn parse_global_section(decoder: &mut Decoder) -> Result<GlobalSection, Error> {
        decoder
            .read_vec(|decoder| {
                let ty = decoder.read_globaltype()?;
                let expr = decoder.read_const_expr()?;
                Ok(Global { ty, expr })
            })
            .map(GlobalSection)
    }

    fn parse_export_section(decoder: &mut Decoder) -> Result<ExportSection, Error> {
        decoder
            .read_vec(|decoder| {
                let name = decoder.read_str()?;
                let kind = decoder.read_u8()?;
                let idx = decoder.read_var_u32()?;
                let kind = match kind {
                    0x00 => ExportKind::Func(idx),
                    0x01 => ExportKind::Table(idx),
                    0x02 => ExportKind::Mem(idx),
                    0x03 => ExportKind::Global(idx),
                    _ => return Err(Error::InvalidExportKind(kind)),
                };
                Ok(Export { name, kind })
            })
            .map(ExportSection)
    }

    fn parse_start_section(decoder: &mut Decoder) -> Result<StartSection, Error> {
        decoder.read_var_u32().map(StartSection)
    }

    fn parse_element_section(decoder: &mut Decoder) -> Result<ElementSection, Error> {
        decoder
            .read_vec(|decoder| {
                let flags = decoder.read_var_u32()?;
                let kind = match flags {
                    0 | 2 | 4 | 6 => {
                        let tabidx = if matches!(flags, 2 | 6) {
                            Some(decoder.read_var_u32()?)
                        } else {
                            None
                        };
                        ElementKind::Active {
                            table: tabidx,
                            offset: decoder.read_const_expr()?,
                        }
                    }
                    1 | 5 => ElementKind::Passive,
                    3 | 7 => ElementKind::Declared,
                    _ => return Err(Error::InvalidFlags(flags, "element segment")),
                };
                let ty = match flags {
                    0 | 4 => RefType::FuncRef,
                    1..=3 => {
                        let elemkind = decoder.read_u8()?;
                        match elemkind {
                            0x00 => RefType::FuncRef,
                            _ => return Err(Error::InvalidElmType(elemkind)),
                        }
                    }
                    5..=7 => decoder.read_reftype()?,
                    _ => return Err(Error::InvalidFlags(flags, "element segment")),
                };
                let init = match flags {
                    0..=3 => decoder.read_svec(|decoder| {
                        decoder
                            .read_var_u32()
                            .map(|idx| ConstExpr(vec![Instruction::RefFunc(idx), Instruction::End]))
                    })?,
                    4..=7 => decoder.read_svec(Decoder::read_const_expr)?,
                    _ => return Err(Error::InvalidFlags(flags, "element segment")),
                };

                Ok(Element { ty, init, kind })
            })
            .map(ElementSection)
    }

    fn parse_code_section(decoder: &mut Decoder) -> Result<CodeSection, Error> {
        decoder
            .read_vec(|decoder| {
                let size = decoder.read_var_u32()?;
                let locals = decoder.read_svec(|decoder| {
                    let n = decoder.read_var_u32()?;
                    let valtype = decoder.read_valtype()?;
                    Ok(Locals { n, ty: valtype })
                })?;
                let expr = decoder.read_expr()?;
                Ok(Code { size, locals, expr })
            })
            .map(CodeSection)
    }

    #[cfg_attr(docsrs, doc(cfg(feature = "parallel")))]
    #[cfg(feature = "parallel")]
    fn par_parse_code_section(decoder: &mut Decoder) -> Result<CodeSection, Error> {
        let codes = decoder.read_vec(|decoder| {
            let size = decoder.read_var_u32()?;
            let start = decoder.reader.position();
            let end = decoder.reader.position() + size as u64;
            decoder.reader.set_position(end);
            Ok((start as usize, end as usize))
        })?;

        use rayon::iter::IntoParallelIterator;
        use rayon::iter::ParallelIterator;
        codes
            .into_par_iter()
            .map(|(start, end)| {
                let mut decoder = Decoder::new(&decoder.reader.get_ref()[start..end]);
                let locals = decoder.read_svec(|decoder| {
                    let n = decoder.read_var_u32()?;
                    let valtype = decoder.read_valtype()?;
                    Ok(Locals { n, ty: valtype })
                })?;
                let expr = decoder.read_expr()?;
                Ok(Code {
                    size: (end - start) as u32,
                    locals,
                    expr,
                })
            })
            .collect::<Result<Vec<_>, _>>()
            .map(CodeSection)
    }

    fn parse_data_section(decoder: &mut Decoder) -> Result<DataSection, Error> {
        decoder
            .read_vec(|decoder| {
                let flags = decoder.read_var_u32()?;
                let kind = match flags {
                    1 => DataKind::Passive,
                    0 | 2 => {
                        let memidx = if flags == 2 {
                            decoder.read_var_u32()?
                        } else {
                            0
                        };
                        let offset = decoder.read_const_expr()?;
                        DataKind::Active {
                            memory: memidx,
                            offset,
                        }
                    }
                    _ => return Err(Error::InvalidFlags(flags, "dats segment")),
                };
                let len = decoder.read_var_u32()? as usize;
                let init = decoder.read_bytes(len)?;

                Ok(Data { init, kind })
            })
            .map(DataSection)
    }

    fn parse_data_count_section(decoder: &mut Decoder) -> Result<DataCountSection, Error> {
        decoder.read_var_u32().map(Some).map(DataCountSection)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_parse() {
        let data = include_bytes!("../tests/spidermonkey.wasm");
        let parser = crate::module::Module::from_bytes(data);
        match parser.parse() {
            Ok(_) => {}
            Err(err) => panic!("{}", err),
        };
    }

    #[cfg(feature = "parallel")]
    #[test]
    fn test_par_parse() {
        let data = include_bytes!("../tests/spidermonkey.wasm");
        let parser = crate::module::Module::from_bytes(data);
        match parser.par_parse() {
            Ok(_) => {}
            Err(err) => panic!("{}", err),
        };
    }
}
