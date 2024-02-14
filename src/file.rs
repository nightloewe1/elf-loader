use alloc::vec::Vec;
use core::cmp::Ordering;
use core::slice;

pub struct ElfFile {
    data: Vec<u8>
}

impl ElfFile {
    pub fn verify_magic(&self) -> bool {
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

    pub fn entry_point_ptr(&self) -> usize {
        usize::from_le_bytes(self.data[24..32].try_into().unwrap())
    }

    /// Returns the program header table pointer, size of an entry and the number of entries
    pub fn program_header_table(&self) -> (usize, u16, u16) {
        (
            usize::from_le_bytes(self.data[32..40].try_into().unwrap()),
            u16::from_le_bytes(self.data[54..56].try_into().unwrap()),
            u16::from_le_bytes(self.data[56..58].try_into().unwrap())
        )
    }

    pub fn program_headers(&self) -> Vec<ProgramHeader> {
        let (ptr, size, num) = self.program_header_table();
        let end = ptr + num as usize * size as usize;

        let mut headers = Vec::with_capacity(num as usize);
        let slice = &self.data[ptr..end];
        let data = unsafe { slice::from_raw_parts(slice.as_ptr() as *const ProgramHeader, num as usize) };

        data.iter().for_each(|header| headers.push(*header));

        headers
    }

    /// Returns a pointer to the data
    pub fn data(&self) -> *const u8 {
        self.data.as_ptr()
    }

    pub fn len(&self) -> usize { self.data.len() }
}

impl From<Vec<u8>> for ElfFile {
    fn from(value: Vec<u8>) -> Self {
        ElfFile {
            data: value
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

#[cfg(test)]
mod tests {
    use std::fs;
    use crate::file::ElfFile;

    #[test]
    fn test_entry_point() {
        let data = fs::read("./kernel").expect("Unable to read test file");
        let elf = ElfFile::from(data);

        assert!(elf.verify_magic());
        assert_eq!(elf.entry_point_ptr(), 0x0);
    }

    #[test]
    fn test_section_headers() {
        let data = fs::read("./kernel").expect("Unable to read test file");
        let elf = ElfFile::from(data);
        let sections = elf.section_headers();

        println!("{}", elf.data.len());
        println!("{:?}", elf.program_header_table());

        let first = &sections[0];
        let header_type = first.header_type;

        unsafe { assert_eq!(header_type, 0x6); }
    }
}