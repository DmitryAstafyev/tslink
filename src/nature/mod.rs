mod composite;
mod primitive;
mod refered;

use crate::{context::Context, error::E};
pub use composite::Composite;
pub use primitive::Primitive;
pub use refered::Refered;
use std::collections::{hash_map::Iter, HashMap};
use syn::{
    punctuated::Punctuated,
    token::{Comma, PathSep},
    FnArg, GenericArgument, Ident, ImplItemFn, ItemFn, Pat, PathArguments, PathSegment, ReturnType,
    Type,
};

pub struct Natures(HashMap<String, Nature>);

impl Natures {
    pub fn new() -> Self {
        Natures(HashMap::new())
    }
    pub fn contains(&self, name: &str) -> bool {
        self.0.contains_key(name)
    }
    pub fn insert(&mut self, name: &str, nature: Nature) -> Result<(), E> {
        if self.contains(name) {
            Err(E::EntityExist(name.to_owned()))
        } else {
            let _ = self.0.insert(name.to_owned(), nature);
            Ok(())
        }
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut Nature> {
        self.0.get_mut(name)
    }

    pub fn filter(&self, filter: fn(&Nature) -> bool) -> Vec<Nature> {
        let mut natures: Vec<Nature> = vec![];
        for (_, n) in self.0.iter() {
            if filter(n) {
                natures.push(n.clone());
            }
        }
        natures
    }

    pub fn iter(&self) -> Iter<'_, std::string::String, Nature> {
        self.0.iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

#[derive(Clone, Debug)]
pub enum Nature {
    Primitive(Primitive),
    Refered(Refered),
    Composite(Composite),
}

impl Nature {
    pub fn is_self_returned(&self) -> bool {
        if let Nature::Composite(Composite::Func(_, out, _)) = self {
            if let Some(Nature::Refered(Refered::Ref(re))) = out.as_deref() {
                re == "Self"
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn bind(&mut self, nature: Nature) -> Result<(), E> {
        match self {
            Self::Primitive(_) => Err(E::Parsing(String::from("Primitive type cannot be bound"))),
            Self::Refered(re) => match re {
                Refered::Struct(_, _, natures) => {
                    natures.push(Box::new(nature));
                    Ok(())
                }
                Refered::Enum(_, _, natures) => {
                    natures.push(Box::new(nature));
                    Ok(())
                }
                Refered::EnumVariant(_, _, natures, _) => {
                    natures.push(Box::new(nature));
                    Ok(())
                }
                _ => Err(E::NotSupported),
            },
            Self::Composite(othr) => match othr {
                composite::Composite::HashMap(k, v) => {
                    if k.is_none() {
                        if let Self::Primitive(p) = nature {
                            let _ = k.insert(p);
                            Ok(())
                        } else {
                            Err(E::Parsing(String::from(
                                "HashMap can use as key only Primitive type",
                            )))
                        }
                    } else if v.is_none() {
                        let _ = v.insert(Box::new(nature));
                        Ok(())
                    } else {
                        Err(E::Parsing(String::from(
                            "HashMap entity already has been bound",
                        )))
                    }
                }
                composite::Composite::Option(o) => {
                    if o.is_some() {
                        Err(E::Parsing(String::from(
                            "Option entity already has been bound",
                        )))
                    } else {
                        let _ = o.insert(Box::new(nature));
                        Ok(())
                    }
                }
                composite::Composite::Tuple(tys) => {
                    tys.push(Box::new(nature));
                    Ok(())
                }
                composite::Composite::Vec(v) => {
                    if v.is_some() {
                        Err(E::Parsing(String::from(
                            "Vec entity already has been bound",
                        )))
                    } else {
                        let _ = v.insert(Box::new(nature));
                        Ok(())
                    }
                }
                _ => Err(E::NotSupported),
            },
        }
    }
}

pub trait Extract<T> {
    fn extract(t: T, context: Context) -> Result<Nature, E>;
}

impl Extract<&GenericArgument> for Nature {
    fn extract(arg: &GenericArgument, context: Context) -> Result<Nature, E> {
        match arg {
            GenericArgument::Type(ty) => Nature::extract(ty, context),
            _ => Err(E::NotSupported),
        }
    }
}

impl Extract<&Ident> for Nature {
    fn extract(ident: &Ident, _context: Context) -> Result<Nature, E> {
        Ok(match ident.to_string().as_str() {
            "u8" | "u16" | "u32" | "i8" | "i16" | "i32" => Nature::Primitive(Primitive::Number),
            "u64" | "i64" => Nature::Primitive(Primitive::BigInt),
            "bool" => Nature::Primitive(Primitive::Boolean),
            "String" => Nature::Primitive(Primitive::String),
            a => Nature::Refered(Refered::Ref(a.to_string())),
        })
    }
}

impl Extract<&Punctuated<PathSegment, PathSep>> for Nature {
    fn extract(segments: &Punctuated<PathSegment, PathSep>, context: Context) -> Result<Nature, E> {
        if segments.len() > 1 {
            return Err(E::Parsing(String::from(
                "Not supported Other Type for more than 1 PathSegment",
            )));
        }
        if let Some(segment) = segments.first() {
            let mut ty = match segment.ident.to_string().as_str() {
                "Vec" => Nature::Composite(composite::Composite::Vec(None)),
                "HashMap" => Nature::Composite(composite::Composite::HashMap(None, None)),
                "Option" => Nature::Composite(composite::Composite::Option(None)),
                _ => {
                    return Err(E::Parsing(String::from(
                        "Only Vec, HashMap and Option are supported",
                    )))
                }
            };
            match &segment.arguments {
                PathArguments::AngleBracketed(args) => {
                    for arg in args.args.iter() {
                        ty.bind(Nature::extract(arg, context.clone())?)?;
                    }
                }
                _ => return Err(E::NotSupported),
            }
            Ok(ty)
        } else {
            Err(E::Parsing(String::from(
                "For not primitive types expected at least one segment",
            )))
        }
    }
}

impl Extract<&Punctuated<Type, Comma>> for Nature {
    fn extract(elements: &Punctuated<Type, Comma>, context: Context) -> Result<Nature, E> {
        let mut ty = Nature::Composite(composite::Composite::Tuple(vec![]));
        for element in elements.iter() {
            ty.bind(Nature::extract(element, context.clone())?)?;
        }
        Ok(ty)
    }
}

impl Extract<&Type> for Nature {
    fn extract(ty: &Type, context: Context) -> Result<Nature, E> {
        match ty {
            Type::Path(type_path) => {
                if let Some(ident) = type_path.path.get_ident() {
                    Nature::extract(ident, context)
                } else {
                    Nature::extract(&type_path.path.segments, context)
                }
            }
            Type::Tuple(type_tuple) => Nature::extract(&type_tuple.elems, context),
            _ => Err(E::NotSupported),
        }
    }
}

impl Extract<Type> for Nature {
    fn extract(ty: Type, context: Context) -> Result<Nature, E> {
        Self::extract(&ty, context)
    }
}

impl Extract<&ImplItemFn> for Nature {
    fn extract(fn_item: &ImplItemFn, context: Context) -> Result<Nature, E> {
        let mut args = vec![];
        for fn_arg in fn_item.sig.inputs.iter() {
            if let FnArg::Typed(ty) = fn_arg {
                let arg_name = if let Pat::Ident(id) = *ty.pat.clone() {
                    id.ident.to_string()
                } else {
                    return Err(E::Parsing(String::from("Cannot find ident for FnArg")));
                };
                args.push(Box::new(Nature::Refered(Refered::FuncArg(
                    arg_name,
                    context.clone(),
                    Box::new(Nature::extract(*ty.ty.clone(), context.clone())?),
                ))));
            }
        }
        let out = match &fn_item.sig.output {
            ReturnType::Default => None,
            ReturnType::Type(_, ty) => Some(Box::new(Self::extract(*ty.clone(), context.clone())?)),
        };
        Ok(Self::Composite(Composite::Func(
            args,
            out,
            fn_item.sig.asyncness.is_some(),
        )))
    }
}

impl Extract<&ItemFn> for Nature {
    fn extract(fn_item: &ItemFn, context: Context) -> Result<Nature, E> {
        let mut args = vec![];
        for fn_arg in fn_item.sig.inputs.iter() {
            if let FnArg::Typed(ty) = fn_arg {
                let arg_name = if let Pat::Ident(id) = *ty.pat.clone() {
                    id.ident.to_string()
                } else {
                    return Err(E::Parsing(String::from("Cannot find ident for FnArg")));
                };
                args.push(Box::new(Nature::Refered(Refered::FuncArg(
                    arg_name,
                    context.clone(),
                    Box::new(Nature::extract(*ty.ty.clone(), context.clone())?),
                ))));
            }
        }
        let out = match &fn_item.sig.output {
            ReturnType::Default => None,
            ReturnType::Type(_, ty) => Some(Box::new(Self::extract(*ty.clone(), context.clone())?)),
        };
        Ok(Self::Composite(Composite::Func(
            args,
            out,
            fn_item.sig.asyncness.is_some(),
        )))
    }
}