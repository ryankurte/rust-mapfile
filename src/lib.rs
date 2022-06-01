use nom::{
    IResult,
    bytes::complete::{take_while, is_not, take_until, take_while1, tag, take_till},
    sequence::{tuple, delimited, terminated, preceded},
    multi::{many0},
    error::{context, VerboseError}, 
    character::{complete::{line_ending, space1, space0, not_line_ending}, is_alphabetic, is_hex_digit, is_alphanumeric}, multi::fold_many0, combinator::{map_res, opt, rest},
};

use nom_supreme::{
    final_parser::{final_parser, Location},
    error::ErrorTree};


use log::{trace, debug};

#[derive(Clone, PartialEq, Debug)]
pub struct MapFile<'a> {
    pub members: Vec<Member<'a>>,
    pub discarded: Vec<Section<'a>>,
    pub memory: Vec<Memory<'a>>,
}

pub fn parse<'a>(s: &'a str) -> Result<MapFile<'a>, ErrorTree<&'a str>> {
    final_parser(MapFile::parse)(s)
}

const MEMORY_CONFIG_TAG: &str = "Memory Configuration";
const LINKER_SCRIPT_TAG: &str = "Linker script and memory map";

impl <'a> MapFile<'a> {
    pub fn parse(s: &'a str) -> IResult<&'a str, Self, ErrorTree<&'a str>> {
        let (o, r) = context(
            "map",
            tuple((
                many0(line_ending),
                tag("Archive member included to satisfy reference by file (symbol)"),
                line_ending,
                many0(Member::parse),
                many0(line_ending),

                tag("Discarded input sections"),
                many0(line_ending),
                many0(terminated(Section::parse, line_ending)),
                many0(line_ending),

                tag("Memory Configuration"),
                many0(line_ending),
                tuple((
                    tag("Name"),
                    space1,
                    tag("Origin"),
                    space1,
                    tag("Length"),
                    space1,
                    tag("Attributes"),
                )),
                many0(line_ending),
                many0(terminated(Memory::parse, line_ending)),
                many0(line_ending),

                tag("Linker script and memory map"),


                // Read the (unhandled) remains of the file
                rest,
            ))
        )(s)?;

        let m = Self {
            members: r.3,
            discarded: r.7,
            memory: r.13,
        };

        println!("Parsed map ({} members, {} discarded, {} sections)", 
            m.members.len(),
            m.discarded.len(),
            m.memory.len(),
        );

        Ok((o, m))
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Member<'a> {
    pub archive: &'a str,
    pub object: &'a str,
    pub symbol: &'a str,
}

impl <'a> Member<'a> {
    pub fn parse(s: &'a str) -> IResult<&'a str, Self, ErrorTree<&'a str>> {
        let (o, r) = context(
            "member",
            tuple((
                take_while1(|c| c != '('),
                delimited(tag("("), is_not(")"), tag(")")),
                line_ending,
                space1,
                take_while1(|c| c != ' '),
                space0,
                delimited(tag("("), is_not(")"), tag(")")),
            ))
        )(s)?;

        Ok((
            o,
            Self{
                archive: r.0,
                object: r.4,
                symbol: r.6,
            },
        ))
    }
}


#[derive(Clone, PartialEq, Debug)]
pub struct Section<'a> {
    pub group: &'a str,
    pub addr: u64,
    pub size: u64,
    pub archive: &'a str,
}

impl <'a> Section<'a> {
    pub fn parse(s: &'a str) -> IResult<&'a str, Self, ErrorTree<&'a str>> {
        let (o, r) = context(
            "section",
            tuple((
                space0,
                take_while1(|c| c != ' '),
                space1,
                map_res(
                    delimited(tag("0x"), is_not(" "), tag(" ")),
                    from_hex,
                ),
                space1,
                map_res(
                    delimited(tag("0x"), is_not(" "), tag(" ")),
                    from_hex,
                ),
                take_while1(|c| c != '\r' && c != '\n'),
            ))
        )(s)?;

        Ok((
            o,
            Self{
                group: r.1,
                addr: r.3,
                size: r.5,
                archive: r.6,
            },
        ))
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Memory<'a> {
    pub name: &'a str,
    pub origin: u64,
    pub length: u64,
    pub attrs: Option<&'a str>,
}

impl <'a> Memory<'a> {
    pub fn parse(s: &'a str) -> IResult<&'a str, Self, ErrorTree<&'a str>> {
        println!("parse: '{}'", s);

        let (o, r) = context(
            "memory",
            tuple((
                take_while1(|c| c != ' '),
                space1,
                map_res(
                    preceded(tag("0x"), take_while1(|c| is_hex_digit(c as u8) )),
                    from_hex,
                ),
                space1,
                map_res(
                    preceded(tag("0x"), take_while1(|c| is_hex_digit(c as u8) )),
                    from_hex,
                ),
                opt(tuple((
                    space1,
                    take_while(|c| c != '\r' && c != '\n'),
                ))),

                //opt(line_ending),
                //rest,
            ))
        )(s)?;

        Ok((
            o,
            Self{
                name: r.0,
                origin: r.2,
                length: r.4,
                attrs: r.5.map(|v| v.1),
            },
        ))
    }
}


fn from_hex(input: &str) -> Result<u64, std::num::ParseIntError> {
    u64::from_str_radix(input, 16)
}

#[cfg(test)]
mod test {
    use std::convert;

    use nom::error::convert_error;

    use super::*;

    const MEMBERS: &[(Member, &str)] = &[
        (
            Member{
                archive: "build/something.a",
                object: "build/something.o",
                symbol: "some_symbol_name",
            },
            r#"build/something.a(something.0.rcgu.o)
            build/something.o (some_symbol_name)"#
        )
    ];

    #[test]
    fn parse_members() {
        for (v, raw) in MEMBERS {
            let (_, p) = Member::parse(raw).unwrap();
            assert_eq!(&p, v);
        }
    }

    const SECTIONS: &[(Section, &str)] = &[
        (
            Section{
                group: ".group",
                addr: 0x01,
                size: 0x0c,
                archive: "build/something.o",
            },
            " .group         0x0000000000000001        0xc build/something.o\r\n",
        ),
    ];

    #[test]
    fn parse_sections() {
        for (v, raw) in SECTIONS {
            let (_, p) = Section::parse(raw).unwrap();
            assert_eq!(&p, v);
        }
    }

    const MEMORIES: &[(Memory, &str)] = &[
        (
            Memory{
                name: "FLASH",
                origin: 0x0000000008040000,
                length: 0x00000000000c0000,
                attrs: Some("xr"),
            },
            "FLASH            0x0000000008040000 0x00000000000c0000 xr",
        ), (
            Memory{
                name: "*default*",
                origin: 0x0000000000000000,
                length: 0xffffffffffffffff,
                attrs: None,
            },
            "*default*        0x0000000000000000 0xffffffffffffffff"
        )
    ];

    #[test]
    fn parse_memories() {
        for (v, raw) in MEMORIES {
            let (_, p) = Memory::parse(raw).unwrap();
            assert_eq!(&p, v);
        }
    }

    const SAMPLE: &str = r#"
Archive member included to satisfy reference by file (symbol)

build/firmware/rust/thumbv7em-none-eabihf/release/libtrezor_lib.a(trezor_lib-626e85b7e122913f.trezor_lib.fd8b2e1b-cgu.0.rcgu.o)
                              build/firmware/embed/extmod/rustmods/modtrezorproto.o (protobuf_type_for_name)
build/firmware/rust/thumbv7em-none-eabihf/release/libtrezor_lib.a(compiler_builtins-50ab11bfb8346963.compiler_builtins.ebf5e920-cgu.0.rcgu.o)
                              build/firmware/vendor/micropython/extmod/moductypes.o (__aeabi_f2d)

Discarded input sections

 .group         0x0000000000000000        0xc build/firmware/embed/extmod/modtrezorconfig/modtrezorconfig.o
 .group         0x0000000000000000        0xd build/firmware/embed/extmod/modtrezorconfig/modtrezorconfig.o
 .group         0x0000000000000000        0xe build/firmware/embed/extmod/modtrezorconfig/modtrezorconfig.o

Memory Configuration

Name             Origin             Length             Attributes
FLASH            0x0000000008040000 0x00000000000c0000 xr
FLASH2           0x0000000008120000 0x00000000000e0000 r
CCMRAM           0x0000000010000000 0x0000000000010000 awl
SRAM             0x0000000020000000 0x0000000000030000 awl
*default*        0x0000000000000000 0xffffffffffffffff

Linker script and memory map

LOAD build/firmware/embed/extmod/modtrezorconfig/modtrezorconfig.o
LOAD build/firmware/vendor/trezor-storage/norcow.o
LOAD build/firmware/vendor/trezor-storage/storage.o
LOAD build/firmware/embed/extmod/trezorobj.o
LOAD build/firmware/embed/extmod/modtrezorcrypto/crc.o
LOAD build/firmware/embed/extmod/modtrezorcrypto/modtrezorcrypto.o
"#;

    #[test]
    fn parse_sample() {
        match super::parse(SAMPLE) {
            Ok(_) => (),
            Err(e) => {
                panic!("{}", e);
            },
        }
    }
}