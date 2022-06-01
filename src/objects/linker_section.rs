use nom::{
    IResult,
    error::context, sequence::{tuple, delimited, terminated}, 
    bytes::complete::{take_while1, tag, is_not}, 
    character::complete::{line_ending, space1, space0},
    combinator::map_res, multi::many0,
};

use nom_supreme::error::ErrorTree;

use crate::from_hex;

#[derive(Clone, PartialEq, Debug)]
pub struct LinkerSection<'a> {
    pub group: &'a str,
    pub addr: u64,
    pub size: u64,
    pub archive: &'a str,
}

impl <'a> LinkerSection<'a> {
    pub fn parse_block(s: &'a str) -> IResult<&'a str, Vec<Self>, ErrorTree<&'a str>> {
        let (o, (_, _, items, _)) = context(
            "discarded sections",
            tuple((
                tag("Discarded input sections"),
                many0(line_ending),
                many0(terminated(LinkerSection::parse_item, line_ending)),
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

    const SECTIONS: &[(LinkerSection, &str)] = &[
        (
            LinkerSection{
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
            let (_, p) = LinkerSection::parse_item(raw).unwrap();
            assert_eq!(&p, v);
        }
    }
}
