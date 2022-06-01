use nom::{
    IResult,
    error::context, sequence::{tuple, delimited, terminated}, 
    bytes::complete::{take_while1, tag, is_not}, 
    character::complete::{line_ending, space1, space0},
    combinator::{map_res, opt}, multi::many0,
};

use nom_supreme::error::ErrorTree;

use crate::{parse_hex, parse_path};
use super::{Symbol};

/// Code section in application binary
#[derive(Clone, PartialEq, Debug)]
pub struct Section<'a> {
    pub name: Option<&'a str>,
    
    pub addr: Option<u64>,
    pub size: Option<u64>,
    pub source: Option<&'a str>,

    pub symbols: Vec<Symbol<'a>>,
}

fn parse_section_header(s: &str) -> IResult<&str, (&str, u64, u64, Option<&str>), ErrorTree<&str>> {
    let (o, (_, name, _, addr, _, size, path, _)) = tuple((
        space0,
        parse_path,
        space1,
        parse_hex,
        space1,
        parse_hex,
        opt(tuple((
            space1,
            parse_path,
        ))),
        line_ending,
    ))(s)?;

    Ok((o, (name, addr, size, path.map(|v| v.1 ))))
}

impl <'a> Section<'a> {
    pub fn parse(s: &'a str) -> IResult<&'a str, Self, ErrorTree<&'a str>> {
        let (o, (_, h1, _, h2, symbols)) = context(
            "section",
            tuple((
                space0,
                // Section label
                opt(parse_section_header),
                opt(delimited(space1, parse_path, line_ending)),
                opt(parse_section_header),
                // Symbols in section
                many0(Symbol::parse),
            ))
        )(s)?;

        println!("section: '\r\n{}\r\n'", s);

        println!("h1: {:#?}, h2: {:#?} symbols: {:#?}", h1, h2, symbols);

        let name = h1.map(|h| h.0 ).or_else(|| h2.map(|h| h.0) );
        let addr = h1.map(|h| h.1 ).or_else(|| h2.map(|h| h.1) );
        let size = h1.map(|h| h.2 ).or_else(|| h2.map(|h| h.2) );
        let source = h2.map(|h| h.3 ).or_else(|| h1.map(|h| h.3) ).flatten();

        Ok((
            o,
            Self{
                name,
                addr,
                size,
                source,
                symbols,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_sections() {

        let sections = &[(
            Section{
                name: Some(".vendorheader"),
                addr: Some(0x0000000008040000),
                size: Some(0xa00),
                source: Some("build/firmware/embed/firmware/vendorheader.o"),
                symbols: vec![
                    Symbol{
                        name: None,
                        addr: 0x0000000008040000,
                        size: None,
                        source: None,
                        value: Some("_binary_embed_vendorheader_vendorheader_unsafe_signed_prod_bin_start"),
                    },
                    Symbol{
                        name: None,
                        addr: 0x0000000008040a00,
                        size: None,
                        source: None,
                        value: Some("_binary_embed_vendorheader_vendorheader_unsafe_signed_prod_bin_end"),
                    },
                ],
            },
            ".vendorheader   0x0000000008040000      0xa00
            *(.vendorheader)
            .vendorheader  0x0000000008040000      0xa00 build/firmware/embed/firmware/vendorheader.o
                            0x0000000008040000                _binary_embed_vendorheader_vendorheader_unsafe_signed_prod_bin_start
                            0x0000000008040a00                _binary_embed_vendorheader_vendorheader_unsafe_signed_prod_bin_end
            "
        ), (
            Section{
                name: None,
                addr: None,
                size: None,
                source: None,
                symbols: vec![
                    Symbol{
                        name: None,
                        addr: 0x0000000020030000,
                        size: None,
                        source: None,
                        value: Some("main_stack_base = (ORIGIN (SRAM) + LENGTH (SRAM))"),
                    },
                    Symbol{
                        name: None,
                        addr: 0x0000000020030000,
                        size: None,
                        source: None,
                        value: Some("_estack = main_stack_base"),
                    }
                ],
            },
            "0x0000000020030000                main_stack_base = (ORIGIN (SRAM) + LENGTH (SRAM))
            0x0000000020030000                _estack = main_stack_base
            "
        )];


        for (v, raw) in sections {
            let (_, p) = Section::parse(raw).unwrap();
            assert_eq!(&p, v);
        }
    }
}