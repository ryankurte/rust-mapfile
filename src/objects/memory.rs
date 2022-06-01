use nom::{
    IResult,
    error::context, sequence::{tuple, delimited, preceded, terminated}, 
    bytes::complete::{take_while1, tag, is_not, take_while}, 
    character::{complete::{line_ending, space1, space0}, is_hex_digit},
    combinator::{map_res, opt}, multi::many0,
};

use nom_supreme::error::ErrorTree;

use crate::from_hex;

#[derive(Clone, PartialEq, Debug)]
pub struct Memory<'a> {
    pub name: &'a str,
    pub origin: u64,
    pub length: u64,
    pub attrs: Option<&'a str>,
}

impl <'a> Memory<'a> {

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
                many0(terminated(Memory::parse_item, line_ending)),
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


#[cfg(test)]
mod test {
    use super::*;

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
            let (_, p) = Memory::parse_item(raw).unwrap();
            assert_eq!(&p, v);
        }
    }
}
