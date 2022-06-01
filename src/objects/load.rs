use nom::{
    IResult,
    error::context, sequence::{tuple}, 
    bytes::complete::{tag}, 
    character::complete::{space1},
};

use nom_supreme::error::ErrorTree;

use crate::{parse_path};

/// File used in linking operation
#[derive(Clone, PartialEq, Debug)]
pub struct File<'a> {
    pub name: &'a str,
}

impl <'a> File<'a> {
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


#[cfg(test)]
mod test {
    use super::*;

    const LOADS: &[(File, &str)] = &[
        (
            File{
                name: "stm32/pendsv.o",
            },
            "LOAD stm32/pendsv.o",
        ),
    ];

    #[test]
    fn parse_loads() {
        for (v, raw) in LOADS {
            let (_, p) = File::parse(raw).unwrap();
            assert_eq!(&p, v);
        }
    }
}
