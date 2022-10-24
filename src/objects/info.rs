use nom::{
    IResult,
    error::context, sequence::{tuple, delimited, terminated, preceded}, 
    bytes::complete::{take_while1, tag, is_not, take_while}, 
    character::complete::{line_ending, space1, space0},
    combinator::{map_res, opt}, multi::many0,
};

use nom_supreme::error::ErrorTree;

use crate::{parse_path, from_hex, is_hex_digit};

/// File used in linking operation
#[derive(Clone, PartialEq, Debug)]
pub struct FileInfo<'a> {
    pub name: &'a str,
}

impl <'a> FileInfo<'a> {
    pub fn parse(s: &'a str) -> IResult<&'a str, Self, ErrorTree<&'a str>> {
        let (o, r) = context(
            "load",
            tuple((
                tag("LOAD"),
                space1,
                parse_path,
            ))
        )(s)?;

        Ok((
            o,
            Self{
                name: r.2,
            },
        ))
    }
}

/// Available memories (from linker file)
#[derive(Clone, PartialEq, Debug)]
pub struct MemoryInfo<'a> {
    pub name: &'a str,
    pub origin: u64,
    pub length: u64,
    pub attrs: Option<&'a str>,
}

impl <'a> MemoryInfo<'a> {

    fn header(s: &'a str) -> IResult<&'a str, (), ErrorTree<&'a str>> {
        let (o, _) = tuple((
            tag("Name"),
            space1,
            tag("Origin"),
            space1,
            tag("Length"),
            space1,
            tag("Attributes"),
        ))(s)?;

        Ok((o, ()))
    }

    pub fn parse_block(s: &'a str) -> IResult<&'a str, Vec<Self>, ErrorTree<&'a str>> {
        let (o, (_, _, _, _, items, _)) = context(
            "memory config",
            tuple((
                terminated(tag("Memory Configuration"), line_ending),
                many0(line_ending),
                Self::header,
                many0(line_ending),
                many0(terminated(MemoryInfo::parse_item, line_ending)),
                many0(line_ending),
            )) 
        )(s)?;

        Ok((o, (items)))
    }

    pub fn parse_item(s: &'a str) -> IResult<&'a str, Self, ErrorTree<&'a str>> {
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


#[derive(Clone, PartialEq, Debug)]
pub struct ArchiveInfo<'a> {
    pub archive: &'a str,
    pub object: &'a str,
    pub symbol: &'a str,
}

impl <'a> ArchiveInfo<'a> {
    pub fn parse_block(s: &'a str) -> IResult<&'a str, Vec<Self>, ErrorTree<&'a str>> {
        let (o, (_, _, items, _)) = context(
            "references",
            tuple((
                space0,
                terminated(tag("Archive member included to satisfy reference by file (symbol)"), line_ending),
                many0(terminated(ArchiveInfo::parse_item, line_ending)),
                many0(line_ending),
            )) 
        )(s)?;

        Ok((o, (items)))
    }

    pub fn parse_item(s: &'a str) -> IResult<&'a str, Self, ErrorTree<&'a str>> {
        let (o, r) = context(
            "reference",
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
pub struct SectionInfo<'a> {
    pub group: &'a str,
    pub addr: u64,
    pub size: u64,
    pub archive: &'a str,
}

impl <'a> SectionInfo<'a> {
    pub fn parse_block(s: &'a str) -> IResult<&'a str, Vec<Self>, ErrorTree<&'a str>> {
        let (o, (_, _, items, _)) = context(
            "discarded sections",
            tuple((
                tag("Discarded input sections"),
                many0(line_ending),
                many0(terminated(SectionInfo::parse_item, line_ending)),
                many0(line_ending),
            )) 
        )(s)?;

        Ok((o, (items)))
    }

    pub fn parse_item(s: &'a str) -> IResult<&'a str, Self, ErrorTree<&'a str>> {
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



#[cfg(test)]
mod test {
    use super::*;

    use pretty_assertions::assert_eq;

    const FILES: &[(FileInfo, &str)] = &[
        (
            FileInfo{
                name: "stm32/pendsv.o",
            },
            "LOAD stm32/pendsv.o",
        ),
    ];

    #[test]
    fn parse_file_info() {
        for (v, raw) in FILES {
            let (_, p) = FileInfo::parse(raw).unwrap();
            assert_eq!(&p, v);
        }
    }

    const MEMORIES: &[(MemoryInfo, &str)] = &[
        (
            MemoryInfo{
                name: "FLASH",
                origin: 0x0000000008040000,
                length: 0x00000000000c0000,
                attrs: Some("xr"),
            },
            "FLASH            0x0000000008040000 0x00000000000c0000 xr",
        ), (
            MemoryInfo{
                name: "*default*",
                origin: 0x0000000000000000,
                length: 0xffffffffffffffff,
                attrs: None,
            },
            "*default*        0x0000000000000000 0xffffffffffffffff"
        )
    ];

    #[test]
    fn parse_memory_info() {
        for (v, raw) in MEMORIES {
            let (_, p) = MemoryInfo::parse_item(raw).unwrap();
            assert_eq!(&p, v);
        }
    }

    const ARCHIVES: &[(ArchiveInfo, &str)] = &[
        (
            ArchiveInfo{
                archive: "build/something.a",
                object: "build/something.o",
                symbol: "some_symbol_name",
            },
            r#"build/something.a(something.0.rcgu.o)
            build/something.o (some_symbol_name)"#
        )
    ];

    #[test]
    fn parse_archive_info() {
        for (v, raw) in ARCHIVES {
            let (_, p) = ArchiveInfo::parse_item(raw).unwrap();
            assert_eq!(&p, v);
        }
    }

    const SECTIONS: &[(SectionInfo, &str)] = &[
        (
            SectionInfo{
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
            let (_, p) = SectionInfo::parse_item(raw).unwrap();
            assert_eq!(&p, v);
        }
    }
}
