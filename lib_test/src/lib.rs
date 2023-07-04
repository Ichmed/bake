use std::collections::HashMap;

use bake::{
    interpolation::{Interpolatable, Interpolate, RuntimeInterpolationError},
    *,
    util::TokenTree
};

#[derive(Debug, Bake)]
pub struct Test {
    pub zahl: u32,
    pub wahrheit: bool,
}

#[derive(Bake)]
pub struct PubTest {
    pub zahl: u32,
    pub wahrheit: bool,
}

#[derive(Bake)]
pub enum EnumTest {
    StructVariant { a: u32, b: bool },
    TupleVariant(u32, bool),
    UnitVarient,
}

#[bake]
pub enum InterpolatedEnumTest {
    StructVariant { a: u32, b: bool },
    TupleVariant(u32, bool),
    UnitVariant,
}

impl InterpolatedEnumTest {
    fn _test(self) {
        match self {
            InterpolatedEnumTest::StructVariant { a: _, b: _ } => todo!(),
            InterpolatedEnumTest::TupleVariant(_, _) => todo!(),
            InterpolatedEnumTest::UnitVariant => todo!(),
        };
    }
}

#[bake]
#[derive(Bake, Debug)]
pub enum InterpolatedEnum {
    Integer(i64),
    String(String),
}

#[derive(Bake, Debug)]
pub struct WithInterpolatedField {
    pub name: String,
    pub value: InterpolatedEnum,
}

impl From<i64> for InterpolatedEnum {
    fn from(value: i64) -> Self {
        InterpolatedEnum::Integer(value.force_fit())
    }
}

impl From<String> for InterpolatedEnum {
    fn from(value: String) -> Self {
        InterpolatedEnum::String(value.force_fit())
    }
}

impl Test {
    pub fn new(wahrheit: bool, zahl: u32) -> Self {
        Self { wahrheit, zahl }
    }
}

// #[derive(Bake)]
pub struct NestTest {
    pub test: Test,
    pub tupel: TupleStruct,
    pub unit: UnitStruct,
}

// #[derive(Bake)]
pub struct TupleStruct(pub u8, pub u32);

#[derive(Bake)]
pub struct UnitStruct;

// Interpolation is off by default
#[derive(Bake, Debug)]
#[bake(interpolate, bake_as(Test))] //Turn on interpolation for all fields
pub struct Ipol {
    pub field_a: u64,
    #[interpolate] //Defaults to true
    pub field_b: u64,
}

pub struct UnbakedUnit;

#[derive(Bake)]
pub struct WithUnbakedUnit(#[bake_via(UnbakedUnit)] pub UnbakedUnit);

// #[cfg(not(feature = "macro"))]
// impl Ipol {
//     fn do_a_thing(&self) {
//         self.field_a.clone() + self.field_b.clone();
//     }
// }

#[derive(Debug, PartialEq)]
pub enum NodeError {
    Interpolation,
    Parsing,
}

impl From<RuntimeInterpolationError> for NodeError {
    fn from(_: RuntimeInterpolationError) -> Self {
        NodeError::Interpolation
    }
}

impl From<syn::Error> for NodeError {
    fn from(_: syn::Error) -> Self {
        NodeError::Parsing
    }
}

#[derive(Bake, Debug, PartialEq)]
#[bake(interpolate, to_tokens)]
pub enum Json {
    Number(i64),
    Boolean(bool),
    String(String),
    List(Vec<Json>),
    Dict(HashMap<String, Box<Json>>)
}

#[cfg(not(feature = "macro"))]
impl Json {
    pub fn truthyness(&self) -> bool{
        match self {
            Json::Number(x) => *x != 0,
            Json::Boolean(x) => *x,
            Json::String(x) => x.len() > 0,
            Json::List(x) => !x.is_empty(),
            Json::Dict(x) => !x.is_empty()
        }
    }
}

impl From<bool> for Json {
    fn from(value: bool) -> Self {
        Json::Boolean(value.force_fit())
    }
}

// #[cfg(not(feature = "macro"))]
pub fn parse(tokens: &str) -> Result<Json, NodeError> {
    Ok(parse_string(tokens)?.fit()?)
}

fn parse_string(tokens: &str) -> Result<Interpolatable<Json>, NodeError> {
    
    let tokens = tokens.trim();
    
    match tokens.chars().peekable().peek() {
        None => Err(NodeError::Parsing),
        Some(c) => match c {
            '[' => Ok(parse_list(tokens)?),
            '$' => parse_interpolation(tokens),
            _ => match tokens {
                "true" => Ok(Json::Boolean(true.force_fit()).fit()?),
                "false" => Ok(Json::Boolean(false.force_fit()).fit()?),
                _ => Err(NodeError::Parsing)
            },
        },
    }
}

fn parse_list(tokens: &str) -> Result<Interpolatable<Json>, NodeError> {
   let tokens = tokens.strip_prefix("[").ok_or(NodeError::Parsing)?;
   let tokens = tokens.strip_suffix("]").ok_or(NodeError::Parsing)?;

   let inner: Result<Vec<_>, _> = tokens.split(",").map(parse_string).collect();
   let result: Interpolatable<Vec<_>> = inner?.into();
   Ok(Json::List(result.fit()?).fit()?)
}

fn parse_interpolation(tokens: &str) -> Result<Interpolatable<Json>, NodeError> {
    let tokens = tokens.strip_prefix("$").ok_or(NodeError::Parsing)?;
    Ok(Interpolatable::<Json>::Interpolation(syn::parse_str::<TokenTree>(tokens)?))
}

pub struct PartialTuple(
    u64,
    #[cfg(feature = "macro")] Interpolatable<bool>,
    #[cfg(not(feature = "macro"))] bool,
);
