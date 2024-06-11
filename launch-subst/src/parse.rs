use crate::types::{Block, Substitution};
use pest::error::{Error, ErrorVariant};
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

macro_rules! bail {
    ($span:expr, $($tt:tt)*) => {
        {
            return Err(Error::new_from_span(
                ErrorVariant::CustomError { message: format!($($tt)*) },
                $span,
            ));
        }
    };
}

pub fn parse(input: &str) -> Result<Vec<Block>, Error<Rule>> {
    let mut pairs = ExprParser::parse(Rule::expr, input).unwrap();
    parse_expr(pairs.next().unwrap())
}

#[derive(Parser)]
#[grammar = "grammar.pest"] // relative to src
struct ExprParser;

fn parse_expr(pair: Pair<Rule>) -> Result<Vec<Block>, Error<Rule>> {
    debug_assert_eq!(pair.as_rule(), Rule::expr);
    pair.into_inner()
        .filter(|pair| pair.as_rule() == Rule::block)
        .map(|pair| parse_block(pair))
        .collect()
}

fn parse_block(pair: Pair<Rule>) -> Result<Block, Error<Rule>> {
    debug_assert_eq!(pair.as_rule(), Rule::block);
    let inner = pair.into_inner().next().unwrap();
    let block = match inner.as_rule() {
        Rule::subst => Block::Substitution(parse_subst(inner)?),
        Rule::text => Block::Text(parse_text(inner)),
        _ => unreachable!(),
    };
    Ok(block)
}

fn parse_text(pair: Pair<Rule>) -> String {
    pair.as_str().to_string()
}

fn parse_subst(pair: Pair<Rule>) -> Result<Substitution, Error<Rule>> {
    let span = pair.as_span();
    let mut inner = pair.into_inner();
    let command = parse_command(inner.next().unwrap());
    let args: Vec<_> = inner.map(|pair| parse_arg(pair)).collect();

    let subst = match command.as_str() {
        "env" => {
            let [var] = args.as_slice() else {
                bail!(span, "expect one argument: ENVIRONMENT_VARIABLE");
            };
            Substitution::Env {
                variable: var.to_string(),
            }
        }
        "optenv" => {
            let (var, default) = match args.as_slice() {
                [var] => (var, None),
                [var, default] => (var, Some(default.to_string())),
                _ => bail!(
                    span,
                    "expect arguments: ENVIRONMENT_VARIABLE [DEFAULT_VALUE]"
                ),
            };

            Substitution::OptEnv {
                variable: var.to_string(),
                default_value: default,
            }
        }
        "find" => {
            let [pkg] = args.as_slice() else {
                bail!(span, "expect one argument: PACKAGE_NAME");
            };

            Substitution::Find {
                pkg: pkg.to_string(),
            }
        }
        "anon" => {
            let [name] = args.as_slice() else {
                bail!(span, "expect one argument: NAME");
            };

            Substitution::Anon {
                name: name.to_string(),
            }
        }
        "arg" => {
            let [name] = args.as_slice() else {
                bail!(span, "expect one argument: ARG_NAME");
            };

            Substitution::Arg {
                name: name.to_string(),
            }
        }
        "eval" => {
            todo!("eval command is not implemented");
        }
        "dirname" => Substitution::DirName,
        _ => Substitution::Other { args },
    };

    Ok(subst)
}

fn parse_command(pair: Pair<Rule>) -> String {
    pair.as_str().to_string()
}

fn parse_arg(pair: Pair<Rule>) -> String {
    pair.as_str().to_string()
}
