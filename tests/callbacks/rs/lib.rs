extern crate tslink;
mod callbacks;

use std::collections::HashMap;

use tslink::tslink;
struct upd {}
#[tslink]
struct Nested {
    pub a: u8,
    pub b: u8,
}
#[tslink]
impl Nested {
    pub fn method_a(&self, abs: u8) -> u8 {
        0
    }
    pub fn method_b(&self, abs: u8) -> u8 {
        0
    }
}
#[tslink]
enum SomeEnum {
    One,
    Two,
    Three(u8),
    Four(u8, u16, u32),
    Five((String, String)),
    Six(Vec<u8>),
    Seven(Nested),
}
#[tslink]
enum FlatEnum {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Nine,
}
#[tslink(class)]
struct StructureA {
    pub _p8: u8,
    pub _p16: u16,
    pub _p32: u32,
    pub _p64: u64,
    pub _a: (u32, u64),
    pub _b: Vec<u64>,
    pub _c: HashMap<String, u64>,
    pub _s: String,
    pub _k: Option<String>,
    pub _y: Nested,
    pub _z: Option<Nested>,
    pub _t: SomeEnum,
}

#[tslink(class)]
struct StructureB {
    pub _p8: u8,
    pub _p16: u16,
    pub _p32: u32,
    pub _p64: u64,
    pub _a: (u32, u64),
    pub _b: Vec<u64>,
    pub _c: HashMap<String, u64>,
    pub _s: String,
    pub _k: Option<String>,
    pub _y: Nested,
    pub _z: Option<Nested>,
    pub _t: SomeEnum,
}

#[tslink]
pub fn testA(a: u8) {
    ()
}

#[tslink]
fn testB(a: StructureA, b: u8) -> u8 {
    0
}

#[tslink]
fn test_generic_fn<G: Fn(i32, i32, bool) -> i32 + Send + 'static>(
    callback: G,
) -> Result<(), String> {
    callback(666, 666, true);
    Ok(())
}

#[tslink]
impl StructureA {
    pub fn method_a(&self, abs: u8) -> u8 {
        0
    }
    #[tslink(ignore)]
    pub fn method_d(&self, abs: u8) -> u8 {
        0
    }
    #[tslink(rename = "methodRenamed")]
    pub async fn method_b(&self) {
        println!(">>>>>>>>>>");
    }

    #[tslink(snake_case_naming)]
    pub fn method_c(&self) {
        println!(">>>>>>>>>>");
    }

    #[tslink]
    fn test_generic_a<F: Fn(i32, i32, bool) -> i32 + Send + 'static>(
        &mut self,
        callback: F,
    ) -> Result<(), String> {
        callback(666, 666, true);
        Ok(())
    }

    #[tslink]
    fn test_generic_b<F>(&mut self, callback: F) -> Result<(), String>
    where
        F: Fn(i32, i32, bool) -> String + Send + 'static,
    {
        callback(666, 666, true);
        Ok(())
    }
}

#[tslink(class)]
struct GenericTest<T: Fn(i32)> {
    pub cb: T,
}

#[tslink(
    target = "./dist/interfaces/interfaces.ts; ./dist/interfaces/interfaces.d.ts",
    ignore = "_p8;_p16;_p32"
)]
struct TestingA<T: Fn(i32)> {
    pub _p8: u8,
    pub _p16: u16,
    pub _p32: u32,
    pub _p64: u64,
    pub _a64: u64,
    pub cb: T,
}
fn main() {
    // let a = StructureA {
    //     _p8: 0,
    //     _p16: 0,
    //     _p32: 0,
    //     _p64: 0,
    //     _a: (0, 0),
    //     _b: vec![],
    //     _c: HashMap::new(),
    //     _s: String::new(),
    //     _k: None,
    //     // _y: "",
    //     // _e: SomeEnum::One,
    // };
    println!("Hello, world!");
}
