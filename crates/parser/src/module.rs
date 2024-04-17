use crate::decode::Decoder;
use crate::parser::ModuleParser;
use crate::section::{
    CodeSection, CustomSection, DataCountSection, DataSection, ElementSection, ExportSection,
    FunctionSection, GlobalSection, ImportSection, MemorySection, StartSection, TableSection,
    TypeSection,
};

#[derive(Debug, Default)]
pub struct Module {
    pub version: u32,
    pub custom_sections: crate::SVec<CustomSection>,
    pub type_section: TypeSection,
    pub import_section: ImportSection,
    pub func_section: FunctionSection,
    pub table_section: TableSection,
    pub memory_section: MemorySection,
    pub global_section: GlobalSection,
    pub export_section: ExportSection,
    pub start_section: StartSection,
    pub element_section: ElementSection,
    pub code_section: CodeSection,
    pub data_section: DataSection,
    pub data_count_section: DataCountSection,
}

impl Module {
    pub fn from_bytes(bytes: &[u8]) -> ModuleParser {
        let decoder = Decoder::new(bytes);

        ModuleParser { decoder }
    }
}
