use nom::{
    IResult,
    error::context, sequence::{tuple, delimited, terminated}, 
    bytes::complete::{take_while1, tag, is_not}, 
    character::complete::{line_ending, space1, space0}, multi::many0,
};

use nom_supreme::error::ErrorTree;


#[derive(Clone, PartialEq, Debug)]
pub struct Reference<'a> {
    pub archive: &'a str,
    pub object: &'a str,
    pub symbol: &'a str,
}

impl <'a> Reference<'a> {
    pub fn parse_block(s: &'a str) -> IResult<&'a str, Vec<Self>, ErrorTree<&'a str>> {
        let (o, (_, _, items, _)) = context(
            "references",
            tuple((
                space0,
                terminated(tag("Archive member included to satisfy reference by file (symbol)"), line_ending),
                many0(terminated(Reference::parse_item, line_ending)),
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

#[cfg(test)]
mod test {
    use super::*;

    const REFERENCES: &[(Reference, &str)] = &[
        (
            Reference{
                archive: "build/something.a",
                object: "build/something.o",
                symbol: "some_symbol_name",
            },
            r#"build/something.a(something.0.rcgu.o)
            build/something.o (some_symbol_name)"#
        )
    ];

    #[test]
    fn parse_references() {
        for (v, raw) in REFERENCES {
            let (_, p) = Reference::parse_item(raw).unwrap();
            assert_eq!(&p, v);
        }
    }
}