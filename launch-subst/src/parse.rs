use crate::types::{Arg, SubstBlock, Substitution};
use itertools::Itertools;
use pest::{error::Error, iterators::Pair, Parser};
use pest_derive::Parser;

// macro_rules! bail {
//     ($span:expr, $($tt:tt)*) => {
//         {
//             return Err(pest::error::Error::new_from_span(
//                 pest::error::ErrorVariant::CustomError { message: format!($($tt)*) },
//                 $span,
//             ));
//         }
//     };
// }

pub fn parse(input: &str) -> Result<Vec<SubstBlock>, Error<Rule>> {
    let mut pairs = ExprParser::parse(Rule::expr, input)?;
    parse_expr(pairs.next().unwrap())
}

#[derive(Parser)]
#[grammar = "grammar.pest"] // relative to src
struct ExprParser;

fn parse_expr(pair: Pair<Rule>) -> Result<Vec<SubstBlock>, Error<Rule>> {
    debug_assert_eq!(pair.as_rule(), Rule::expr);
    pair.into_inner()
        .take_while(|pair| pair.as_rule() == Rule::block) // Stop at Rule::EOI
        .map(|pair| parse_block(pair))
        .collect()
}

fn parse_block(pair: Pair<Rule>) -> Result<SubstBlock, Error<Rule>> {
    debug_assert_eq!(pair.as_rule(), Rule::block);
    let inner = pair.into_inner().next().unwrap();
    let block = match inner.as_rule() {
        Rule::subst => SubstBlock::Substitution(parse_subst(inner)?),
        Rule::text => SubstBlock::Text(parse_text(inner)),
        _ => unreachable!(),
    };
    Ok(block)
}

fn parse_text(pair: Pair<Rule>) -> String {
    pair.as_str().to_string()
}

fn parse_subst(pair: Pair<Rule>) -> Result<Substitution, Error<Rule>> {
    debug_assert_eq!(pair.as_rule(), Rule::subst);
    let mut inner = pair.into_inner();
    let command = parse_command(inner.next().unwrap());
    let args: Vec<_> = inner.map(|pair| parse_arg(pair)).try_collect()?;
    Ok(Substitution { command, args })
}

fn parse_command(pair: Pair<Rule>) -> String {
    pair.as_str().to_string()
}

fn parse_arg(pair: Pair<Rule>) -> Result<Arg, Error<Rule>> {
    debug_assert_eq!(pair.as_rule(), Rule::arg);
    let blocks: Vec<_> = pair
        .into_inner()
        .map(|pair| parse_block(pair))
        .try_collect()?;
    Ok(Arg(blocks))
}
