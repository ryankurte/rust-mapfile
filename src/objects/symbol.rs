
use nom::{
    IResult,
    error::context, sequence::{tuple, preceded}, 
    bytes::complete::{take_while1}, 
    character::complete::{line_ending, space1, space0, multispace0},
    combinator::{opt},
};

use nom_supreme::error::ErrorTree;

use crate::{parse_hex, parse_path};

/// A symbol included in the application binary
#[derive(Clone, PartialEq, Debug)]
pub struct Symbol<'a> {
    pub name: Option<&'a str>,
    pub addr: u64,
    pub size: Option<u64>,
    pub source: Option<&'a str>,
    pub value: Option<&'a str>,
}

impl <'a> Symbol<'a> {
    /// Create a new symbol with an address (and other fields empty)
    pub const fn new(addr: u64) -> Self {
        Self{
            addr,
            name: None,
            size: None,
            source: None,
            value: None,
        }
    }

    pub fn parse(s: &'a str) -> IResult<&'a str, Self, ErrorTree<&'a str>> {

        let parse_label = tuple((
            line_ending,
            space1,
            parse_hex,
            space0,
            parse_path,
        ));

        let parse_object = tuple((
            parse_hex,
            space0,
            parse_path,
            opt(parse_label),
            opt(line_ending),
        ));

        let (o, (_, addr, _, obj, val, _)) = context(
            "symbol",
            tuple((
                space0,
                parse_hex,
                space1,
                opt(parse_object),
                opt(preceded(multispace0, take_while1(|c| c != '\r' && c != '\n' ))),
                opt(line_ending),
            ))
        )(s)?;

        Ok((
            o,
            Self{
                name: obj.map(|v| v.3.map(|w| w.4 )).flatten(),
                addr,
                size: obj.map(|v| v.0 ),
                source: obj.map(|v| v.2 ),
                value: val,
            },
        ))
    }
}


#[cfg(test)]
mod test {
    use super::*;

    const SYMBOLS: &[(Symbol, &str)] = &[
        (
            Symbol{
                name: Some("norcow_set"),
                addr: 0x0000000008042108,
                size: Some(0x30),
                source: Some("build/firmware/vendor/trezor-storage/norcow.o"),
                value: None,
            },
            "    0x0000000008042108       0x30 build/firmware/vendor/trezor-storage/norcow.o
                0x0000000008042108                norcow_set"
        ), (
            Symbol{
                addr: 0x0000000020030000,
                size: None,
                source: None,
                name: None,
                value: Some("main_stack_base = (ORIGIN (SRAM) + LENGTH (SRAM))")
            },
            "    0x0000000020030000                main_stack_base = (ORIGIN (SRAM) + LENGTH (SRAM))"
        ), (
            Symbol{
                addr: 0x0000000008040a00,
                size: None,
                source: None,
                name: None,
                value: Some("_binary_embed_vendorheader_vendorheader_unsafe_signed_prod_bin_end")
            },
            "    0x0000000008040a00                _binary_embed_vendorheader_vendorheader_unsafe_signed_prod_bin_end"
        ), (
            Symbol{
                addr: 0x00000000080fde08,
                size: None,
                source: None,
                name: None,
                value: Some("data_lma = LOADADDR (.data)")
            },
            "    0x00000000080fde08                data_lma = LOADADDR (.data)"
        ),

        
    ];

    #[test]
    fn parse_symbols() {
        for (v, raw) in SYMBOLS {
            let (_, p) = Symbol::parse(raw).unwrap();
            assert_eq!(&p, v);
        }
    }
}
