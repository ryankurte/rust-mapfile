
use nom::{
    IResult,
    error::context, sequence::{tuple, preceded}, 
    bytes::complete::{take_while1, tag, take_until1}, 
    character::complete::{line_ending, space1, space0, multispace0, one_of, newline},
    combinator::{opt, map}, branch::alt, multi::{many1, many0},
};

use nom_supreme::error::ErrorTree;

use crate::{parse_hex, parse_path};

/// A symbol included in the application binary
#[derive(Clone, PartialEq, Debug)]
pub struct Symbol<'a> {
    pub name: Option<&'a str>,
    pub addr: u64,
    pub kind: SymbolKind<'a>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SymbolKind<'a> {
    Value(&'a str),
    Object{
        size: u64,
        source: Option<&'a str>,
    },
}

impl <'a> Symbol<'a> {
    pub fn parse(s: &'a str) -> IResult<&'a str, Self, ErrorTree<&'a str>> {

        println!("PARSE: '{}'", s);

        // Start by parsing the first line of the object
        let (mut o, (indent, addr, _, kind)) = context(
            "symbol",
            tuple((
                get_indent,
                parse_hex,              // Address
                space1,
                // From here we either have a value or a size + file
                alt((
                    map(tuple((
                        parse_hex,       // Size
                        space0,
                        opt(parse_path), // File path (TODO: may be on next line)
                        line_ending,
                    )), |(size, _, source, _)| SymbolKind::Object{ size, source } ),

                    map(tuple((
                        take_while1(|c| c != '\r' && c != '\n'),
                    )), |v| SymbolKind::Value(v.0) ),
                )),
            ))
        )(s)?;

        println!("indent: {} kind: {:#08x?}", indent, kind);

        // Then, check whether the next line is relevant
        let r = context(
            "symbol name",
            tuple((
                space1,
                parse_hex,
                space1,
                parse_path,
            ))
        )(o);

        let mut name = None;

        match r {
            Ok((o1, (_, addr1, _, val1))) if addr1 == addr => {
                // Set function name
                name = Some(val1);

                println!("Found name: {}", val1);

                // Update remainder
                o = o1;
            },
            Ok((o1, (_, addr1, _, val1))) => {

                println!("Found attribute: 0x%{:x} {}", addr1, val1);
                // TODO

                // Update remainder
                o = o1;
            }
            _ => (),
        }

        Ok((
            o,
            Self{
                name,
                addr,
                kind,
            },
        ))
    }


    pub fn parse_many(s: &'a str) -> IResult<&'a str, Vec<Symbol>, ErrorTree<&'a str>> {

        let (rem, output) = context("symbols", 
            many0(tuple((
                many0(newline),
                Symbol::parse,
                many0(newline),
            )))
        )(s)?;

        let symbols = output.iter().map(|v| v.1.clone() ).collect();

        Ok((rem, symbols))
    }

}

/// Calculate indentation level
pub fn get_indent<'a>(s: &'a str) -> IResult<&'a str, usize, ErrorTree<&'a str>> {
    // Fetch indentation characters
    let (o, spaces) = context(
        "indentation",
        take_while1(|c| c == ' ' || c == '\t')
    )(s)?;

    let mut n = 0;
    for s in spaces.chars() {
        n += match s {
            ' ' => 1,
            '\t' => 4,
            _ => 0,
        }
    }

    Ok((o, n))
}

#[cfg(test)]
mod test {
    use super::*;

    use nom::multi::many0;
    use pretty_assertions::assert_eq;

    const SYMBOLS: &[(Symbol, &str)] = &[
        (
            Symbol{
                name: Some("norcow_set"),
                addr: 0x0000000008042108,
                kind: SymbolKind::Object{
                    size: 0x30,
                    source: Some("build/firmware/vendor/trezor-storage/norcow.o"),
                },
            },
"   0x0000000008042108       0x30 build/firmware/vendor/trezor-storage/norcow.o
    0x0000000008042108                norcow_set
"
        ), (
            Symbol{
                addr: 0x0000000020030000,
                name: None,
                kind: SymbolKind::Value("main_stack_base = (ORIGIN (SRAM) + LENGTH (SRAM))")
            },
"    0x0000000020030000                main_stack_base = (ORIGIN (SRAM) + LENGTH (SRAM))"
        ), (
            Symbol{
                addr: 0x0000000008040a00,
                name: None,
                kind: SymbolKind::Value("_binary_embed_vendorheader_vendorheader_unsafe_signed_prod_bin_end")
            },
" 0x0000000008040a00                _binary_embed_vendorheader_vendorheader_unsafe_signed_prod_bin_end
"
        ), (
            Symbol{
                addr: 0x00000000080fde08,
                name: None,
                kind: SymbolKind::Value("data_lma = LOADADDR (.data)")
            },
" 0x00000000080fde08                data_lma = LOADADDR (.data)
"
        ), (
            Symbol{
                addr: 0x0000000008120000,
                name: None,
                kind: SymbolKind::Object{
                    size: 0xf9d8,
                    source: Some("build/firmware/frozen_mpy.o"),
                },
            },
" 0x0000000008120000     0xf9d8 build/firmware/frozen_mpy.o
                         0xff3c (size before relaxing))
"
        ),
    ];

    #[test]
    fn parse_symbols() {
        for (v, raw) in SYMBOLS {
            let (_, p) = Symbol::parse(raw).unwrap();
            assert_eq!(&p, v);
        }
    }

    #[test]
    fn parse_chained_symbols() {

        let raw = 
" 0x0000000008040000                _binary_embed_vendorheader_vendorheader_unsafe_signed_prod_bin_start
  0x0000000008040a00                _binary_embed_vendorheader_vendorheader_unsafe_signed_prod_bin_end
";

        let v = vec![
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
        ];

        let p = Symbol::parse_many(raw).unwrap();

        assert_eq!(p.1, v);

    }

}
