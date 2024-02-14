use std::fs::OpenOptions;
use std::io::Read;
use elf_loader::ElfFile;

fn main() {
    let mut file = OpenOptions::new().read(true).open("./elffile").expect("Unable to open file");
    let mut data = Vec::new();

    file.read_to_end(&mut data).expect("Unable to read file");
    println!("Magic is {:X?}", &data[0..4]);

    let elf = ElfFile::from(data);

    println!("Magic: {:?}", elf.verify_magic());
}