use super::ir::SkSLSymbolTable::SymbolTable;
// Source: src/sksl/SkSLMangler.h:18-18
// Source: src/sksl/SkSLMangler.h:30-30
pub struct Mangler {
    fCounter: i32,
}
impl Default for Mangler {
    fn default() -> Self {
        Self { fCounter: 0 }
    }
}
impl Mangler {
    // Source: src/sksl/SkSLMangler.h:25-27
    pub fn reset(&mut self) {
        self.fCounter = 0;
    }
    // Source: src/sksl/SkSLMangler.cpp:21-74
    // Adapted byte view must retain readable C++ preconditions for '_' lookahead;
    // caller supplies original accessible backing for any terminator/lookahead beyond the view.
    pub unsafe fn uniqueName(&mut self, mut baseName: &[u8], symbolTable: &SymbolTable) -> Vec<u8> {
        if baseName.first() == Some(&b'$') {
            baseName = &baseName[1..];
        }
        if baseName.first() == Some(&b'_') {
            let mut offset = 1;
            while baseName.as_ptr().add(offset).read().is_ascii_digit() {
                offset += 1;
            }
            if offset > 1
                && baseName.as_ptr().add(offset).read() == b'_'
                && baseName.as_ptr().add(offset + 1).read() != 0
            {
                baseName = &baseName[offset + 1..];
            } else {
                baseName = &baseName[1..];
            }
        }
        let mut uniqueName = [0u8; 256];
        uniqueName[0] = b'_';
        loop {
            // Selected decimal primitive writes directly into the original stack buffer;
            // no additional String owner/allocation is introduced by the Rust adapter.
            use std::io::Write;
            let counter = self.fCounter;
            self.fCounter = self
                .fCounter
                .checked_add(1)
                .expect("native signed counter must not overflow");
            let mut sink = &mut uniqueName[1..];
            write!(sink, "{}", counter).expect("native signed decimal fits buffer");
            let mut end = 256 - sink.len();
            uniqueName[end] = b'_';
            end += 1;
            let copy = baseName.len().min(uniqueName.len() - end);
            uniqueName[end..end + copy].copy_from_slice(&baseName[..copy]);
            end += copy;
            let view = &uniqueName[..end];
            if symbolTable.find(view).is_none() {
                return view.to_vec();
            }
        }
    }
}
