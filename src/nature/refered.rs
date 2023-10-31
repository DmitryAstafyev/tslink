use crate::{
    context::Context,
    error::E,
    nature::{Nature, VariableTokenStream},
};
use proc_macro2::TokenStream;
use std::ops::Deref;
use quote::{quote, format_ident};

#[derive(Clone, Debug)]
pub enum Refered {
    Struct(String, Context, Vec<Box<Nature>>),
    Enum(String, Context, Vec<Box<Nature>>),
    // name, context, values, is_flat
    EnumVariant(String, Context, Vec<Box<Nature>>, bool),
    Func(String, Context, Box<Nature>),
    Field(String, Context, Box<Nature>),
    FuncArg(String, Context, Box<Nature>),
    Ref(String),
}

impl Refered {
    pub fn is_flat_varians(variants: &Vec<Box<Nature>>) -> Result<bool, E> {
        for variant in variants {
            if let Nature::Refered(Refered::EnumVariant(_, _, values, _)) = variant.deref() {
                if !values.is_empty() {
                    return Ok(false);
                }
            } else {
                return Err(E::Parsing(String::from("Given Nature isn't enum varian")));
            }
        }
        Ok(true)
    }

    pub fn is_enum_flat(&self) -> Result<bool, E> {
        if let Refered::Enum(_, _, variants) = self {
            Refered::is_flat_varians(&variants)
        } else {
            Err(E::Parsing(String::from("Given Nature isn't enum")))
        }
    }
}

impl VariableTokenStream for Refered {
    fn token_stream(&self, var_name: &str) -> Result<TokenStream, E> {
        let var_name = format_ident!("{}", var_name);
        match self {   
            Self::Ref(_) => {
                Ok(quote!{
                    serde_json::to_string(&#var_name).map_err(|e| e.to_string())?
                })
            },
            _ => {
                Err(E::Parsing(format!("Only reference to entity (struct / enum) can be convert into JSON string (var: {var_name})")))
            }
        }
    }
}
