use core::cmp::Ordering;
use core::slice;

pub struct ElfFile<'a> {
    data: &'a mut [u8],
}

impl<'a> ElfFile<'a> {
    pub fn read(buffer: &'a mut [u8]) -> ElfFile<'a> {
        ElfFile {
            data: buffer
        }
    }

    /// Returns true if the elf file magic is valid
    pub fn is_valid(&self) -> bool {
        if self.data.len() < 4 {
            return false;
        }

        for (first, second) in [0x7Fu8, 0x45, 0x4C, 0x46].iter().zip(self.data[0..4].iter()) {
            if first.cmp(second) != Ordering::Equal {
                return false;
            }
        }

        true
    }

    /// Returns the entrypoint address
    pub fn entrypoint(&self) -> usize {
        usize::from_le_bytes(self.data[24..32].try_into().unwrap())
    }

    /// Returns the program header table pointer, size of an entry and the number of entries
    fn program_header_table(&self) -> (usize, u16, u16) {
        (
            usize::from_le_bytes(self.data[32..40].try_into().unwrap()),
            u16::from_le_bytes(self.data[54..56].try_into().unwrap()),
            u16::from_le_bytes(self.data[56..58].try_into().unwrap())
        )
    }

    /// Returns a slice of the program headers
    pub fn program_headers(&self) -> &[ProgramHeader] {
        let (ptr, size, num) = self.program_header_table();
        let end = ptr + num as usize * size as usize;

        let slice = &self.data[ptr..end];
        let data = unsafe { slice::from_raw_parts(slice.as_ptr() as *const ProgramHeader, num as usize) };

        data
    }

    /// Returns the section header table pointer, size of an entry and the number of entries
    fn section_header_table(&self) -> (usize, u16, u16) {
        (
            usize::from_le_bytes(self.data[40..48].try_into().unwrap()),
            u16::from_le_bytes(self.data[58..60].try_into().unwrap()),
            u16::from_le_bytes(self.data[60..62].try_into().unwrap())
        )
    }

    /// Returns a slice of the section headers
    pub fn section_headers(&self) -> &[SectionHeader] {
        let (ptr, size, num) = self.section_header_table();
        let end = ptr + num as usize * size as usize;

        let slice = &self.data[ptr..end];
        let data = unsafe { slice::from_raw_parts(slice.as_ptr() as *const SectionHeader, num as usize) };

        data
    }

    /// Returns a reference to the elf file data
    pub fn data(&'a self) -> &'a [u8] {
        return self.data
    }

    /// Returns the length in bytes required to load all loadable segments into memory
    pub fn load_segments_len(&self) -> usize {
        let program_headers = self.program_headers();

        let mut start = usize::MAX;
        let mut end = usize::MIN;

        for header in program_headers {
            let segment_end = header.v_addr + header.memory_size;

            if (header.v_addr as usize) < start {
                start = header.v_addr as usize;
            }

            if (segment_end as usize) > end {
                end = segment_end as usize;
            }
        }

        end - start
    }


    /// Loads all PT_LOAD segments into memory starting at base
    pub fn load(&self, base: &mut [u8]) {
        let program_headers = self.program_headers();
        let file_data = self.data();

        for header in program_headers {
            if header.header_type != 0x1 {
                continue;
            }

            let start = header.v_addr as usize;

            let mut ptr = &mut base[start] as *mut u8;
            let start_file = header.offset as usize;
            let end_file = start_file + header.memory_size as usize;

            for i in start_file..end_file {
                unsafe {
                    *ptr = file_data[i];
                    ptr = ptr.add(1);
                }
            }
        }
    }

    /// Applies the relocations necessary for the elf file to work
    pub fn relocate(&self, base: &mut [u8]) {
        let section_headers = self.section_headers();

        for header in section_headers {
            if header.header_type != 0x4 {
                continue;
            }

            let start_file = self.data.as_ptr() as usize + header.offset as usize;
            let num = header.size as usize / header.entry_size as usize;

            let sections = unsafe { slice::from_raw_parts(start_file as *mut RelocationSection, num) };

            for section in sections {
                unsafe {
                    let ptr = base.as_mut_ptr().add(section.offset) as *mut u64;
                    *ptr = base.as_ptr() as u64 + section.addend as u64;
                }
            }
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(packed)]
pub struct ProgramHeader {
    pub header_type: u32,
    pub flags: u32,
    pub offset: u64,
    pub v_addr: u64,
    pub p_addr: u64,
    pub file_size: u64,
    pub memory_size: u64,
    pub align: u64
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(packed)]
pub struct SectionHeader {
    pub name_offset: u32,
    pub header_type: u32,
    pub flags: u64,
    pub addr: u64,
    pub offset: u64,
    pub size: u64,
    pub link: u32,
    pub info: u32,
    pub addr_align: u64,
    pub entry_size: u64
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(packed)]
pub struct RelocationSection {
    pub offset: usize,
    pub info: usize,
    pub addend: usize,
}

#[cfg(test)]
mod tests {
    use std::fs;
    use crate::file::ElfFile;

    #[test]
    fn test_entry_point() {
        let data = fs::read("./kernel").expect("Unable to read test file");
        let elf = ElfFile::read(data);

        assert!(elf.is_valid());
    }
}