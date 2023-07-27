use std::collections::HashMap;

use bake::interpolation::{Interpolate, IntoInterpolation, RuntimeInterpolationError};
use bake::{bake, interpolation::{Interpolatable, FlattenInterpolation}, Bake, Bakeable};
use nom::branch::alt;
use nom::character::complete::{alphanumeric0, alphanumeric1, char, digit1};
use nom::combinator::map_res;
use nom::multi::{many0, separated_list0};
use nom::sequence::delimited;
use nom::IResult;

use parse_hyperlinks::take_until_unbalanced;

#[derive(Bake, Debug, PartialEq)]
#[bake(to_tokens)]
pub enum Json {
    Number(i64),
    Boolean(bool),
    String(String),
    #[interpolate]
    List(Vec<Json>),
    #[interpolate]
    Dict(HashMap<String, Json>),
}

impl Json {
    pub fn as_json(&self) -> String {
        match self {
            Json::Number(n) => n.to_string(),
            Json::Boolean(b) => b.to_string(),
            Json::String(s) => format!("{:?}", s.to_string()),
            Json::List(list) => {
                let inner: Vec<_> = list.iter().map(Json::as_json).collect();
                format!("[{}]", inner.join(", ")).to_owned()
            }
            Json::Dict(dict) => {
                let inner: Vec<_> = dict
                    .iter()
                    .map(|(key, value)| format!("\"{key}\": {}", value.as_json()))
                    .collect();
                format!("{{ {} }}", inner.join(", ")).to_owned()
            }
        }
    }
}

impl From<i64> for Json {
    fn from(value: i64) -> Self {
        Self::Number(value)
    }
}

impl From<bool> for Json {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<&str> for Json {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

#[cfg(not(feature = "macro"))]
impl<T> From<Vec<T>> for Json
where
    T: Into<Json>,
{
    fn from(value: Vec<T>) -> Self {
        Self::List(value.into_iter().map(|x| x.into()).collect::<Vec<Json>>())
    }
}

#[derive(Debug)]
pub enum JsonError {
    Syntax,
    RuntimeInterpolation(RuntimeInterpolationError),
}

impl From<RuntimeInterpolationError> for JsonError {
    fn from(value: RuntimeInterpolationError) -> Self {
        JsonError::RuntimeInterpolation(value)
    }
}

#[cfg(not(feature = "macro"))]
pub fn parse(input: &str) -> Result<Json, JsonError> {
    use nom::combinator::all_consuming;

    let (_, parsed) = all_consuming(parse_json_node)(input).map_err(|_| JsonError::Syntax)?;

    Ok(parsed.fit()?)
}

pub fn parse_json_node(i: &str) -> IResult<&str, Interpolatable<Json>> {
    let (i, _) = whitespace(i)?;
    alt((
        parse_interpolation,
        parse_dict,
        parse_list,
        parse_number,
        parse_string,
    ))(i)
}

fn parse_number(i: &str) -> IResult<&str, Interpolatable<Json>> {
    let (i, number) = map_res(digit1, str::parse)(i)?;

    Ok((i, Interpolatable::Actual(Json::Number(number))))
}

fn parse_string(i: &str) -> IResult<&str, Interpolatable<Json>> {
    let (i, _) = char('"')(i)?;
    let (i, number) = alphanumeric0(i)?;
    let (i, _) = char('"')(i)?;

    Ok((i, Json::String(number.to_owned()).interpolate()))
}

fn parse_interpolation(i: &str) -> IResult<&str, Interpolatable<Json>> {
    let (i, _) = char('$')(i)?;
    let (i, _) = whitespace(i)?;
    let (i, tree) = delimited(char('{'), take_until_unbalanced('{', '}'), char('}'))(i)?;
    let (i, _) = whitespace(i)?;

    let tree = bake::util::parse_str(tree).expect("syntax error");

    Ok((i, Interpolatable::Inter(tree)))
}

fn parse_list(i: &str) -> IResult<&str, Interpolatable<Json>> {
    let (i, list) = delimited(
        char('['),
        separated_list0(char(','), parse_json_node),
        char(']'),
    )(i)?;

    Ok((i, Json::List(list.flatten_interpolation().nom(i)?).interpolate()))
}

fn parse_dict(i: &str) -> IResult<&str, Interpolatable<Json>> {
    let (i, dict) = delimited(
        char('{'),
        separated_list0(char(','), parse_dict_entry),
        char('}'),
    )(i)?;

    // Do not use `flatten_interpolation()` so we do not need to construct another Hashmap first
    let dict: Interpolatable<HashMap<String, Json>> = dict.into_iter().collect();

    Ok((i, Json::Dict(dict.nom(i)?).interpolate()))
}

fn parse_dict_entry(i: &str) -> IResult<&str, Interpolatable<(String, Json)>> {
    let (i, _) = whitespace(i)?;
    let (i, _) = char('"')(i)?;
    let (i, key) = alphanumeric1(i)?;
    let (i, _) = char('"')(i)?;
    let (i, _) = whitespace(i)?;
    let (i, _) = char(':')(i)?;
    let (i, _) = whitespace(i)?;
    let (i, value) = parse_json_node(i)?;
    let (i, _) = whitespace(i)?;

    let pair = (key.to_owned().interpolate(), value).flatten_interpolation();

    Ok((i, pair))
}

fn whitespace(i: &str) -> IResult<&str, ()> {
    let (i, _) = many0(alt((char(' '), char('\t'), char('\n'))))(i)?;
    Ok((i, ()))
}

#[test]
fn test() {
    // let x =
    //     parse_json_node("{\"liste\" : ${ { let x = 10 ; x } }}").unwrap();

    let i = "{\n\"a\" : 10, \"lol\" : 10}";

    let x = parse(i).unwrap();
    println!("{:?}", x);
}
