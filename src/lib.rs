use nom::{
    IResult,
    bytes::complete::{tag, take_while1},
    sequence::{tuple, terminated, preceded, delimited},
    multi::{many0},
    error::{context}, 
    character::{complete::{line_ending, space1, space0}, is_hex_digit}, combinator::{rest, map_res, opt},
};

use nom_supreme::{
    final_parser::{final_parser, Location},
    error::ErrorTree
};


use log::{trace, debug};

pub mod objects;
use objects::*;

/// Linker .map file object for parsing
#[derive(Clone, PartialEq, Debug)]
pub struct MapFile<'a> {
    pub references: Vec<ArchiveInfo<'a>>,
    pub discarded: Vec<SectionInfo<'a>>,
    pub memory: Vec<MemoryInfo<'a>>,
    pub files: Vec<FileInfo<'a>>,
    pub sections: Vec<Object<'a>>,
}

/// Map file information
#[derive(Clone, PartialEq, Debug)]
pub struct MapInfo {
    pub num_members: usize,
    pub num_memories: usize,
    pub num_files: usize,
    pub num_sections: usize,
}



impl <'a> MapFile<'a> {

    pub fn parse(s: &'a str) -> Result<Self, ErrorTree<&'a str>> {
        final_parser(MapFile::parse_internal)(s)
    }

    pub fn info(&self) -> MapInfo {
        MapInfo{
            num_members: self.references.len(),
            num_memories: self.memory.len(),
            num_files: self.files.len(),
            num_sections: self.sections.len(),
        }
    }

    fn parse_internal(s: &'a str) -> IResult<&'a str, Self, ErrorTree<&'a str>> {
        let (o, (_, _, references, discarded, memory, _,files, sections, rest)) = context(
            "map",
            tuple((
                space0,
                opt(line_ending),
                ArchiveInfo::parse_block,
                SectionInfo::parse_block,
                MemoryInfo::parse_block,
                
                terminated(tag("Linker script and memory map"), many0(line_ending)),
                many0(terminated(FileInfo::parse, line_ending)),

                many0(delimited(space0, Object::parse, line_ending)),

                // Read the (unhandled) remains of the file
                rest,
            ))
        )(s)?;

        println!("remainder: {:#?}", rest);


        let m = Self {
            references,
            discarded,
            memory,
            files,
            sections,
        };

        println!("Parsed map ({} refs, {} discarded, {} memories, {} files, {} sections)", 
            m.references.len(),
            m.discarded.len(),
            m.memory.len(),
            m.files.len(),
            m.sections.len(),
        );

        Ok((o, m))
    }
}



fn from_hex(input: &str) -> Result<u64, std::num::ParseIntError> {
    u64::from_str_radix(input, 16)
}

fn parse_hex(s: &str) -> IResult<&str, u64, ErrorTree<&str>> {
    context(
        "hex",
        map_res(
            preceded(tag("0x"), take_while1(|c| is_hex_digit(c as u8) )),
            from_hex,
        )
    )(s)
}

fn parse_path(s: &str) -> IResult<&str, &str, ErrorTree<&str>> {
    context(
        "path",
        take_while1(|c| c != ' ' && c != '\r' && c != '\n')
    )(s)
}
