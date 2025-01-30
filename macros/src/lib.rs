use std::hash::{Hasher as _, Hash};

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse::Parse, Lit};

macro_rules! assert_unique_feature {
    () => {};
    ($first:tt $(,$rest:tt)*) => {
        $(
            #[cfg(all(feature = $first, feature = $rest))]
            compile_error!(concat!("features \"", $first, "\" and \"", $rest, "\" cannot be used together"));
        )*
        assert_unique_feature!($($rest),*);
    }
}
assert_unique_feature!("ahash");

#[cfg(feature = "ahash")]
type Hasher = ahash::AHasher;

struct IdInput {
    name: String
}

fn stringify_stream(input: &syn::parse::ParseStream) -> syn::Result<String> {
    Ok(if input.peek(syn::Ident) {
        input.parse::<syn::Ident>()?.to_string()
    } else if input.peek(syn::Lit) {
        match input.parse() {
            Ok(Lit::Str(s)) => s.value(),
            Ok(Lit::ByteStr(s)) => match std::str::from_utf8(&s.value()) {
                Ok(it) => it.to_string(),
                Err(_) => return Err(syn::Error::new(s.span(), "id must be a valid utf-8 string"))
            },
            Ok(Lit::CStr(s)) => match s.value().into_string() {
                Ok(it) => it,
                Err(_) => return Err(syn::Error::new(s.span(), "id must be a valid utf-8 string"))
            },
            Ok(Lit::Byte(b)) => match std::str::from_utf8(&[b.value()]) {
                Ok(it) => it.to_string(),
                Err(_) => return Err(syn::Error::new(b.span(), "byte not a valid utf-8 character"))
            },
            Ok(Lit::Char(c)) => c.value().to_string(),
            Ok(Lit::Int(int)) => int.base10_digits().to_string(),
            Ok(Lit::Float(f)) => {
                // There's no sane way to handle this:
                // - there's several ways to write the same float value
                // - some values in the source code will differ to what is actually stored due to rounding errors
                //   - which will cause unexpected behaviors as compile time and runtime floats will be differently handled
                //   - formatting floats to strings is also not an option because it will differ from actual input
                return Err(syn::Error::new(f.span(), "can't make id from floats due to non-injective source->value mapping"));
            },
            Ok(Lit::Bool(b)) => if b.value() {
                "true".to_string()
            } else {
                "false".to_string()
            },
            Ok(other) => {
                return Err(syn::Error::new(other.span(), "can't make id from this literal"));
            }
            _ => unreachable!("unexpected value instead of literal")
        }
    } else if input.peek(syn::Lifetime) {
        let lifetime: syn::Lifetime = input.parse()?;
        format!("'{}", lifetime.ident.to_string())
    } else {
        return Err(input.error("unsupported id macro value"));
    })
}

impl Parse for IdInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut name = stringify_stream(&input)?;
        while !input.is_empty() {
            name.push(' ');
            name.push_str(stringify_stream(&input)?.as_str());
        }

        Ok(IdInput {
            name
        })
    }
}

/// Macro that produces a constant [`NameId`] value at compile time.
/// 
/// It's equivalent to calling `name_id::NameId::from_raw(hash, name)` where
/// hash is the appropriate hash value for `name`.
/// 
/// When used with `name-id` crate, this macro will inherit and use the same
/// hashing algorithm as specified with crate features (`ahash` being the
/// default).
/// 
#[cfg_attr(not(feature = "_nested_doc"), doc = "[`NameId`]: #")]
#[cfg_attr(feature = "_nested_doc", doc = "[`NameId`]: ./struct.NameId.html")]
#[proc_macro]
pub fn id(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as IdInput);
    let ident = input.name;
    let mut hasher = Hasher::default();
    ident.hash(&mut hasher);
    let hash = hasher.finish();
    let entry = if cfg!(debug_assertions) {
        quote! {
            name_id::NameId::from_raw(#hash, #ident)
        }
    } else {
        quote! {
            name_id::NameId::from_raw(#hash)
        }
    };
    entry.into()
}
