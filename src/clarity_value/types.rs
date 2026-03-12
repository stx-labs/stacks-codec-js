use lazy_static::lazy_static;
use regex::Regex;
use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::fmt;
use std::io::Write;
use std::ops::Deref;

use crate::address::c32::c32_address;
use crate::hex::{encode_hex, encode_hex_no_prefix};

use super::deserialize::TypePrefix;

pub const MAX_STRING_LEN: u8 = 128;
pub const MAX_VALUE_SIZE: u32 = 1024 * 1024; // 1MB
                                             // this is the charged size for wrapped values, i.e., response or optionals

pub const CONTRACT_MIN_NAME_LENGTH: usize = 1;
pub const CONTRACT_MAX_NAME_LENGTH: usize = 40;

#[derive(Debug, PartialEq)]
pub struct ClarityValue {
    pub serialized_bytes: Option<Vec<u8>>,
    pub value: Value,
}

impl ClarityValue {
    pub fn new_with_bytes<T: AsRef<[u8]>>(serialized_bytes: T, value: Value) -> ClarityValue {
        ClarityValue {
            serialized_bytes: Some(serialized_bytes.as_ref().to_vec()),
            value,
        }
    }

    pub fn new(value: Value) -> ClarityValue {
        ClarityValue {
            serialized_bytes: None,
            value,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Value {
    Int(i128),
    UInt(u128),
    Bool(bool),
    Buffer(Vec<u8>),
    List(Vec<ClarityValue>),
    StringUTF8(Vec<Vec<u8>>),
    StringASCII(Vec<u8>),
    PrincipalStandard(StandardPrincipalData),
    PrincipalContract(QualifiedContractIdentifier),
    Tuple(BTreeMap<ClarityName, ClarityValue>),
    OptionalSome(Box<ClarityValue>),
    OptionalNone,
    ResponseOk(Box<ClarityValue>),
    ResponseErr(Box<ClarityValue>),
}

impl Value {
    pub fn string_utf8(bytes: Vec<u8>) -> Value {
        let validated_utf8_str = String::from_utf8_lossy(&bytes);
        let mut data = vec![];
        for char in validated_utf8_str.chars() {
            let mut encoded_char: Vec<u8> = vec![0; char.len_utf8()];
            char.encode_utf8(&mut encoded_char[..]);
            data.push(encoded_char);
        }
        Value::StringUTF8(data)
    }

    pub fn type_prefix(&self) -> TypePrefix {
        use Value::*;
        match self {
            Int(_) => TypePrefix::Int,
            UInt(_) => TypePrefix::UInt,
            Bool(true) => TypePrefix::BoolTrue,
            Bool(false) => TypePrefix::BoolFalse,
            PrincipalStandard(_) => TypePrefix::PrincipalStandard,
            PrincipalContract(_) => TypePrefix::PrincipalContract,
            ResponseOk(_) => TypePrefix::ResponseOk,
            ResponseErr(_) => TypePrefix::ResponseErr,
            OptionalSome(_) => TypePrefix::OptionalSome,
            OptionalNone => TypePrefix::OptionalNone,
            Tuple(_) => TypePrefix::Tuple,
            Buffer(_) => TypePrefix::Buffer,
            List(_) => TypePrefix::List,
            StringASCII(_) => TypePrefix::StringASCII,
            StringUTF8(_) => TypePrefix::StringUTF8,
        }
    }

    pub fn repr_string(&self) -> String {
        let mut w: Vec<u8> = Vec::new();
        Value::repr_string_to_buffer(self, &mut w).unwrap();
        let string_result = unsafe { String::from_utf8_unchecked(w) };
        string_result
    }

    fn repr_string_to_buffer(val: &Value, w: &mut Vec<u8>) -> std::io::Result<()> {
        use Value::*;
        match val {
            Int(data) => write!(w, "{}", data),
            UInt(data) => write!(w, "u{}", data),
            Bool(data) => write!(w, "{}", data),
            OptionalSome(data) => {
                write!(w, "(some ")?;
                Value::repr_string_to_buffer(&data.value, w)?;
                write!(w, ")")
            }
            OptionalNone => write!(w, "none"),
            ResponseOk(data) => {
                write!(w, "(ok ")?;
                Value::repr_string_to_buffer(&data.value, w)?;
                write!(w, ")")
            }
            ResponseErr(data) => {
                write!(w, "(err ")?;
                Value::repr_string_to_buffer(&data.value, w)?;
                write!(w, ")")
            }
            Tuple(data) => {
                write!(w, "(tuple")?;
                for (name, value) in data.iter() {
                    write!(w, " ({} ", name)?;
                    Value::repr_string_to_buffer(&value.value, w)?;
                    write!(w, ")")?;
                }
                write!(w, ")")
            }
            PrincipalStandard(data) => {
                write!(w, "'{}", c32_address(data.0, &data.1).unwrap())
            }
            PrincipalContract(data) => {
                write!(
                    w,
                    "'{}.{}",
                    c32_address(data.issuer.0, &data.issuer.1).unwrap(),
                    data.name
                )
            }
            Buffer(value) => {
                write!(w, "{}", encode_hex(value))
            }
            List(value) => {
                write!(w, "(list")?;
                for val in value {
                    write!(w, " ")?;
                    Value::repr_string_to_buffer(&val.value, w)?;
                }
                write!(w, ")")
            }
            StringASCII(data) => {
                write!(w, "\"")?;
                for c in data.iter() {
                    write!(w, "{}", std::ascii::escape_default(*c))?;
                }
                write!(w, "\"")
            }
            StringUTF8(data) => {
                write!(w, "u\"")?;
                for c in data.iter() {
                    if c.len() > 1 {
                        // We escape extended charset
                        write!(w, "\\u{{{}}}", encode_hex_no_prefix(c))?;
                    } else {
                        // We render an ASCII char, escaped
                        write!(w, "{}", std::ascii::escape_default(c[0]))?;
                    }
                }
                write!(w, "\"")
            }
        }
    }

    pub fn type_signature(&self) -> String {
        let mut w: Vec<u8> = Vec::new();
        Value::type_signature_to_buffer(self, &mut w).unwrap();
        let string_result = unsafe { String::from_utf8_unchecked(w) };
        string_result
    }

    fn type_signature_to_buffer(val: &Value, w: &mut Vec<u8>) -> std::io::Result<()> {
        use Value::*;
        match val {
            Int(_) => write!(w, "int"),
            UInt(_) => write!(w, "uint"),
            Bool(_) => write!(w, "bool"),
            OptionalSome(data) => {
                write!(w, "(optional ")?;
                Value::type_signature_to_buffer(&data.value, w)?;
                write!(w, ")")
            }
            OptionalNone => write!(w, "(optional UnknownType)"),
            ResponseOk(data) => {
                write!(w, "(response ")?;
                Value::type_signature_to_buffer(&data.value, w)?;
                write!(w, " UnknownType)")
            }
            ResponseErr(data) => {
                write!(w, "(response UnknownType ")?;
                Value::type_signature_to_buffer(&data.value, w)?;
                write!(w, ")")
            }
            Tuple(data) => {
                write!(w, "(tuple")?;
                for (name, value) in data.iter() {
                    write!(w, " ({} ", name)?;
                    Value::type_signature_to_buffer(&value.value, w)?;
                    write!(w, ")")?;
                }
                write!(w, ")")
            }
            PrincipalStandard(_) | PrincipalContract(_) => write!(w, "principal"),
            Buffer(value) => write!(w, "(buff {})", value.len()),
            List(value) => {
                write!(w, "(list {} ", value.len())?;
                if value.len() > 0 {
                    // TODO: this should use the least common supertype
                    Value::type_signature_to_buffer(&value[0].value, w)?;
                } else {
                    write!(w, "UnknownType")?;
                }
                write!(w, ")")
            }
            StringASCII(data) => {
                write!(w, "(string-ascii {})", data.len())
            }
            StringUTF8(data) => {
                write!(w, "(string-utf8 {})", data.len() * 4)
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StandardPrincipalData(pub u8, pub [u8; 20]);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QualifiedContractIdentifier {
    pub issuer: StandardPrincipalData,
    pub name: ClarityName,
}

#[macro_export]
macro_rules! guarded_string {
    ($Name:ident, $Label:literal, $Regex:expr) => {
        #[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $Name(pub String);
        impl TryFrom<String> for $Name {
            type Error = String;
            fn try_from(value: String) -> Result<Self, Self::Error> {
                if value.len() > (MAX_STRING_LEN as usize) {
                    return Err(format!("Bad name value {}, {}", $Label, value));
                }
                if $Regex.is_match(&value) {
                    Ok(Self(value))
                } else {
                    Err(format!("Bad name value {}, {}", $Label, value))
                }
            }
        }

        impl Clone for $Name {
            fn clone(&self) -> Self {
                Self(self.0.to_string())
            }
        }

        impl $Name {
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl Deref for $Name {
            type Target = str;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl Borrow<str> for $Name {
            fn borrow(&self) -> &str {
                self.as_str()
            }
        }

        impl Into<String> for $Name {
            fn into(self) -> String {
                self.0
            }
        }

        impl From<&'_ str> for $Name {
            fn from(value: &str) -> Self {
                Self::try_from(value.to_string()).unwrap()
            }
        }

        impl fmt::Display for $Name {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

lazy_static! {
    pub static ref CLARITY_NAME_REGEX: Regex =
        Regex::new("^[a-zA-Z]([a-zA-Z0-9]|[-_!?+<>=/*])*$|^[-+=/*]$|^[<>]=?$").unwrap();
    pub static ref CONTRACT_NAME_REGEX: Regex =
        Regex::new("^[a-zA-Z]([a-zA-Z0-9]|[-_])*$|^__transient$").unwrap();
}

guarded_string!(ClarityName, "ClarityName", CLARITY_NAME_REGEX);
guarded_string!(ContractName, "ContractName", CONTRACT_NAME_REGEX);
