use std::fs::OpenOptions;
use std::io::Read;
use std::slice;
use elf_loader::{ElfFile, RelocatableSection};

fn main() {
    let mut file = OpenOptions::new().read(true).open("./kernel").expect("Unable to open file");
    let mut data = Vec::new();

    file.read_to_end(&mut data).expect("Unable to read file");
    println!("Magic is {:X?}", &data[0..4]);

    let elf = ElfFile::read(&mut data);

    println!("Magic: {:?}", elf.is_valid());

    elf.section_headers().iter().for_each(|header| {
        let header_type = header.header_type;
        let offset = header.offset;
        let size = header.size;
        let entry_size = header.entry_size;
        println!("{} {:#016X} - {:#016X} - {}", header_type, offset, size, entry_size)
    });

    let hdr = elf.section_headers();

    for header in hdr {
        if header.header_type != 0x4 {
            continue;
        }

        let start_file = elf.data().as_ptr() as usize + header.offset as usize;
        let num = header.size as usize / header.entry_size as usize;

        let sections = unsafe { slice::from_raw_parts(start_file as *mut RelocatableSection, num) };

        for section in sections {
            let offset = section.offset;
            let info = section.info;
            let addend = section.addend;
            println!("{:#016X}: {:#016X} = {:#16X}", offset, info, addend)
        }
    }
}