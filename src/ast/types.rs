#[allow(dead_code)]
pub(crate) enum PrimitiveType {
    UInt8, // Unsigned ints
    UInt16,
    UInt32,
    UInt64,
    UInt128,
    SInt8, // Signed ints
    SInt16,
    SInt32,
    SInt64,
    SInt128,
    // ------ //
    Bool,
    Char,
    Unit, // ()
    RefStaticStr // Do we allow this?
}

pub(crate) enum Ty {
    Primitive(PrimitiveType), // u8, bool, etc...
    Userdefined(String), // For example: "pack SomeType"
    Generic(String, Box<Ty>), // For example: Hashmap<i32, Vec<Something, Allocator>>
    Ref(Box<Ty>), // &Ty
    Ptr(Box<Ty>), // *Ty or even ** Ty
}

