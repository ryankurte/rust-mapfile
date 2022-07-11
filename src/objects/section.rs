use nom::{
    IResult,
    error::context, sequence::{tuple, delimited, terminated, preceded}, 
    bytes::complete::{take_while1, tag, is_not, take_until1}, 
    character::complete::{line_ending, space1, space0},
    combinator::{map_res, opt}, multi::many0,
};

use nom_supreme::error::ErrorTree;

use crate::{parse_hex, parse_path};
use super::{Symbol, SymbolKind};

/// Memories included in output binary
#[derive(Clone, PartialEq, Debug)]
pub struct Section<'a> {
    pub name: Option<&'a str>,
    pub addr: Option<u64>,
    pub size: Option<u64>,

    pub sections: Vec<Object<'a>>,
}

impl <'a> Section<'a> {
    pub fn parse(s: &'a str) -> IResult<&'a str, Self, ErrorTree<&'a str>> {

        let (o, (header, body)) = context(
            "section",
            tuple((
                // Memory header (name, location, size), all but first section
                opt(Self::parse_section_header),
                // Memory contents (text), all indented
                many0(Object::parse),
            ))
        )(s)?;

        println!("SECTION body: {:#08x?}, remainder: {}", body, o);

        Ok((o, Self{
            name: header.map(|h| h.0 ),
            addr: header.map(|h| h.1 ),
            size: header.map(|h| h.2 ),
            sections: vec![],
        }))
    }

    fn parse_section_header(s: &str) -> IResult<&str, (&str, u64, u64), ErrorTree<&str>> {
        let (o, (name, _, addr, _, size, _)) = tuple((
            parse_path, // name (ie. `.flash`)
            space1,
            parse_hex,  // address
            space1,
            parse_hex,  // size (used?)
            line_ending,
        ))(s)?;
    
        Ok((o, (name, addr, size)))
    }
}

/// Code section in application binary
#[derive(Clone, PartialEq, Debug)]
pub struct Object<'a> {
    pub name: Option<&'a str>,
    
    pub addr: Option<u64>,
    pub size: Option<u64>,
    pub source: Option<&'a str>,

    pub symbols: Vec<Symbol<'a>>,
}


impl <'a> Object<'a> {
    pub fn parse(s: &'a str) -> IResult<&'a str, Self, ErrorTree<&'a str>> {
        // Sections (except the first) _should_ start with a 
        // name (`.something`) and no indent, with nested sections and symbols indented below this
        let (o, (label, header, symbols)) = context(
            "object",
            tuple((
                // Object section label (ie. `*(.vector_table)`)
                opt(delimited(space1, parse_path, line_ending)),
                // Section header (name, location, size), all but first section
                opt(Self::parse_object_header),
                // Section contents (text)
                many0(Symbol::parse),
            ))
        )(s)?;

        println!("section: '\r\n{}\r\n'", s);

        println!("label: {:#?} header: {:#?}, symbols: {:#?}", label, header, symbols);


        Ok((
            o,
            Self{
                name: header.map(|h| h.0 ),
                addr: header.map(|h| h.1 ),
                size: header.map(|h| h.2 ),
                source: header.map(|h| h.3 ).flatten(),
                symbols,
            },
        ))
    }


    fn parse_object_header(s: &str) -> IResult<&str, (&str, u64, u64, Option<&str>), ErrorTree<&str>> {
        let (o, (_, name, _, addr, _, size, file, _)) = tuple((
            space1,
            parse_path, // Section name
            space1,
            parse_hex,  // Section address
            space1,
            parse_hex,  // Section size
            opt(tuple((
                space1,
                parse_path, // File name
            ))),
            line_ending,
        ))(s)?;

        Ok((o, (name, addr, size, file.map(|v| v.1 ))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_objects() {

        let sections = &[(
            Object{
                name: Some(".vendorheader"),
                addr: Some(0x0000000008040000),
                size: Some(0xa00),
                source: Some("build/firmware/embed/firmware/vendorheader.o"),
                symbols: vec![
                    Symbol{
                        name: None,
                        addr: 0x0000000008040000,
                        kind: SymbolKind::Value("_binary_embed_vendorheader_vendorheader_unsafe_signed_prod_bin_start"),
                    },
                    Symbol{
                        name: None,
                        addr: 0x0000000008040a00,
                        kind: SymbolKind::Value("_binary_embed_vendorheader_vendorheader_unsafe_signed_prod_bin_end"),
                    },
                ],
            },
" *(.vendorheader)
  .vendorheader  0x0000000008040000      0xa00 build/firmware/embed/firmware/vendorheader.o
                0x0000000008040000                _binary_embed_vendorheader_vendorheader_unsafe_signed_prod_bin_start
                0x0000000008040a00                _binary_embed_vendorheader_vendorheader_unsafe_signed_prod_bin_end
            "
        ), (
            Object{
                name: None,
                addr: None,
                size: None,
                source: None,
                symbols: vec![
                    Symbol{
                        name: None,
                        addr: 0x0000000020030000,
                        kind: SymbolKind::Value("main_stack_base = (ORIGIN (SRAM) + LENGTH (SRAM))"),
                    },
                    Symbol{
                        name: None,
                        addr: 0x0000000020030000,
                        kind: SymbolKind::Value("_estack = main_stack_base"),
                    }
                ],
            },
            "0x0000000020030000                main_stack_base = (ORIGIN (SRAM) + LENGTH (SRAM))
            0x0000000020030000                _estack = main_stack_base
            "
        ), (
            Object{
                name: Some(".flash2"),
                addr: Some(0x0000000008120000),
                size: Some(0x62a00),
                source: Some("build/firmware/frozen_mpy.o(.rodata*)"),
                symbols: vec![
                    Symbol{
                        name: Some(".rodata.str1.1"),
                        addr: 0x0000000008120000,
                        kind: SymbolKind::Object{
                            size: 0xf9d8,
                            source: Some("0xf9d8 build/firmware/frozen_mpy.o"),
                        },
                    }, Symbol{
                        name: Some(".rodata"),
                        addr: 0x000000000812f9d8,
                        kind: SymbolKind::Object{
                            size: 0x91e1,
                            source: Some("build/firmware/frozen_mpy.o"),
                        },
                    }, Symbol{
                        name: Some("*fill*"),
                        addr: 0x0000000008138bb9,
                        kind: SymbolKind::Object{
                            size: 0x1,
                            source: None,
                        },
                    }
                ],
            },
" build/firmware/frozen_mpy.o(.rodata*)
 .rodata.str1.1
                0x0000000008120000     0xf9d8 build/firmware/frozen_mpy.o
                                       0xff3c (size before relaxing)
 .rodata        0x000000000812f9d8     0x91e1 build/firmware/frozen_mpy.o
 *fill*         0x0000000008138bb9        0x1"
        )];


        for (v, raw) in sections {
            let (_, p) = Object::parse(raw).unwrap();
            assert_eq!(&p, v);
        }
    }
}
