#[derive(Clone, PartialEq, Debug)]
pub struct Load<'a> {
    pub name: Option<&'a str>,
    pub addr: u64,
    pub size: Option<u64>,
}

impl <'a> Load<'a> {
    pub fn parse(s: &'a str) -> IResult<&'a str, Self, ErrorTree<&'a str>> {
        let (o, r) = context(
            "load",
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
