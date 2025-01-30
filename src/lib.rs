#![doc = include_str!("../doc/crate.md")]

#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

use core::hash::{Hash, Hasher as _};

#[cfg(feature = "detect_collisions")]
use alloc::collections::BTreeMap;

pub use name_id_macros::id;

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

#[cfg(feature = "detect_collisions")]
static mut LOOKUP: BTreeMap<u64, &'static str> = BTreeMap::new();
#[cfg(feature = "detect_collisions")]
fn lookup() -> &'static mut BTreeMap<u64, &'static str> {
    unsafe { core::ptr::addr_of_mut!(LOOKUP).as_mut().unwrap_unchecked() }
}

/// A small identifier type based on string hash values.
/// 
/// String identifiers are hashed using
#[cfg_attr(feature = "ahash", doc = "[`ahash`](ahash)")]
/// hasher, and stored as a `u64`.
/// 
/// For convenient compile-time constuction use [`id!`][id] macro.
#[derive(Clone, Copy)]
#[cfg_attr(
    all(not(feature = "fixed_size"), not(all(debug_assertions, feature = "debug_name"))),
    repr(C)
)]
#[cfg_attr(any(not(all(debug_assertions, feature = "debug_name")), feature = "fixed_size"), repr(transparent))]
pub struct NameId {
    value: u64,
    #[cfg(all(debug_assertions, feature = "debug_name"))]
    name: &'static str,
    #[cfg(all(not(debug_assertions), not(feature = "debug_name"), feature = "fixed_size"))]
    _padding: [u8; core::mem::size_of::<&'static str>()],
}

impl NameId {
    pub const SIZE: usize = core::mem::size_of::<Self>();

    /// Creates a new `NameId` using one of supported input data types. This
    /// constructor can be extended by implementing [`Into<NameId>`] for
    /// external types.
    /// 
    /// Use [`from_raw`] for constant-time construction where hash is known, or
    /// [`id!`][id] macro for computing the hash value from string.
    /// 
    /// [`from_raw`]: NameId::from_raw
    #[inline(always)]
    pub fn new<T: Into<Self>>(name: T) -> Self {
        name.into()
    }

    /// Constructs a `NameId` from hash `value`.
    #[cfg(not(feature = "debug_name"))]
    pub const fn from_raw(value: u64) -> Self {
        Self {
            value,
            #[cfg(feature = "fixed_size")]
            _padding: [0; core::mem::size_of::<&'static str>()],
        }
    }

    /// Constructs a `NameId` from hash `value` and a debug `label`.
    #[cfg(feature = "debug_name")]
    pub const fn from_raw(value: u64, label: &'static str) -> Self {
        #[cfg(debug_assertions)]
        {Self { value, name: label }}
        #[cfg(all(not(debug_assertions), feature = "fixed_size"))]
        {Self { value, _padding: [0; core::mem::size_of::<&'static str>()] }}
        #[cfg(not(any(debug_assertions, feature = "fixed_size")))]
        {Self { value }}
    }

    /// Returns the raw hash value.
    pub const fn value(&self) -> u64 {
        self.value
    }

    /// Checks whether two `NameId`s are equal.
    #[inline(always)]
    pub const fn const_eq(&self, other: &Self) -> bool {
        self.const_eq_value(other.value)
    }

    /// Same as [`const_eq`][NameId::const_eq], but accepts a hash/id value directly.
    #[inline]
    pub const fn const_eq_value(&self, other: u64) -> bool {
        self.value == other
    }

    /// Returns [`Ordering`][core::cmp::Ordering] of two `NameId`s.
    #[inline(always)]
    pub const fn const_cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.const_cmp_value(other.value)
    }
    
    /// Same as [`const_cmp`][NameId::const_cmp], but accepts a hash/id value directly.
    pub const fn const_cmp_value(&self, other: u64) -> core::cmp::Ordering {
        if self.value > other {
            core::cmp::Ordering::Greater
        } else if self.value == other {
            core::cmp::Ordering::Equal
        } else {
            core::cmp::Ordering::Less
        }
    }
}

/// Use [`const_eq`][NameId::const_eq] to perform equality checks in const
/// contexts.
impl PartialEq for NameId {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.const_eq(other)
    }
}

/// Use [`const_eq`][NameId::const_eq] to perform equality checks in const
/// contexts.
impl Eq for NameId {}
impl<S: AsRef<str>> PartialEq<S> for NameId {
    /// Compares hash of `other` to the hash stored by this `NameId`.
    fn eq(&self, other: &S) -> bool {
        let mut hasher = Hasher::default();
        other.as_ref().hash(&mut hasher);
        let value = hasher.finish();
        self.value.eq(&value)
    }
}

/// Use [`const_cmp`][NameId::const_cmp] to perform comparison in const
/// contexts.
impl PartialOrd for NameId {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Use [`const_cmp`][NameId::const_cmp] to perform comparison in const
/// contexts.
impl Ord for NameId {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.const_cmp(other)
    }
}
impl core::hash::Hash for NameId {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.value)
    }
}

macro_rules! specialize_signature {
    ((&'static str) => |$name: ident| $it: block) => {
        #[allow(unreachable_code)]
        impl From<&'static str> for NameId {
            fn from($name: &'static str) -> Self {
                $it
            }
        }
    };
    ((&'a $($T: tt)*) => |$name: ident| $it: block) => {
        #[allow(unreachable_code)]
        impl <'a> From<&'a $($T)*> for NameId {
            fn from($name: &'a $($T)*) -> Self {
                $it
            }
        }
    };
    ((alloc::borrow::Cow<'a, str>) => |$name: ident| $it: block) => {
        #[allow(unreachable_code)]
        impl <'a> From<alloc::borrow::Cow<'a, str>> for NameId {
            fn from($name: alloc::borrow::Cow<'a, str>) -> Self {
                $it
            }
        }
    };
    (($($T: tt)*) => |$name: ident| $it: block) => {
        #[allow(unreachable_code)]
        impl From<$($T)*> for NameId {
            fn from($name: $($T)*) -> Self {
                $it
            }
        }
    };
}
macro_rules! specialize_to_debug_name {
    ($name: ident : &'static str) => {$name};
    ($name: ident : &'a alloc::string::String) => {$name.clone().leak()};
    ($name: ident : alloc::string::String) => {$name.clone().leak()};
    ($name: ident : &'a alloc::borrow::Cow<'a, str>) => {alloc::string::ToString::to_string($name).leak()};
    ($name: ident : alloc::borrow::Cow<'a, str>) => {alloc::string::ToString::to_string(&$name).leak()};
    ($name: ident : &'a core::ffi::CStr) => {alloc::string::ToString::to_string(&$name.to_string_lossy()).leak()};
    ($name: ident : &'a alloc::ffi::CString) => {alloc::string::ToString::to_string(&$name.to_string_lossy()).leak()};
    ($name: ident : alloc::ffi::CString) => {alloc::string::ToString::to_string(&$name.to_string_lossy()).leak()};
    ($name: ident : &'a [u8]) => {alloc::string::ToString::to_string(&alloc::string::String::from_utf8_lossy($name)).leak()};
    ($name: ident : &'a alloc::vec::Vec<u8>) => {alloc::string::ToString::to_string(&alloc::string::String::from_utf8_lossy($name)).leak()};
    ($name: ident : alloc::vec::Vec<u8>) => {alloc::string::ToString::to_string(&alloc::string::String::from_utf8_lossy(&$name)).leak()};
    ($name: ident : $($T: tt)*) => {$name};
}
macro_rules! impl_from {
    ($($T: tt)*) => {
        specialize_signature!(($($T)*) => |name| {
            let mut hasher = Hasher::default();
            name.hash(&mut hasher);
            let value = hasher.finish();
            #[cfg(feature = "detect_collisions")]
            {
                let name = specialize_to_debug_name!(name: $($T)*);
                if let Some(previous) = lookup().get(&value) {
                    assert_eq!(
                        *previous,
                        name,
                        "hash id collision: {} collides with {}",
                        previous,
                        name,
                    );
                }
                lookup().insert(value, name);
                #[cfg(feature = "debug_name")]
                return Self::from_raw(value, name);
            }
            #[cfg(not(feature = "debug_name"))]
            return NameId::from_raw(value);
            #[cfg(feature = "debug_name")]
            return NameId::from_raw(value, specialize_to_debug_name!(name: $($T)*));
        });
    };
}

impl_from!(&'static str);
#[cfg(feature = "alloc")]
impl_from!(&'a alloc::string::String);
#[cfg(feature = "alloc")]
impl_from!(alloc::string::String);
#[cfg(feature = "alloc")]
impl_from!(&'a alloc::borrow::Cow<'a, str>);
#[cfg(feature = "alloc")]
impl_from!(alloc::borrow::Cow<'a, str>);

#[cfg(any(not(all(debug_assertions, feature = "debug_name")), all(debug_assertions, feature = "debug_name", feature = "alloc")))]
impl_from!(&'a core::ffi::CStr);
#[cfg(feature = "alloc")]
impl_from!(&'a alloc::ffi::CString);
#[cfg(feature = "alloc")]
impl_from!(alloc::ffi::CString);

#[cfg(any(not(all(debug_assertions, feature = "debug_name")), all(debug_assertions, feature = "debug_name", feature = "alloc")))]
impl_from!(&'a [u8]);
#[cfg(feature = "alloc")]
impl_from!(&'a alloc::vec::Vec<u8>);
#[cfg(feature = "alloc")]
impl_from!(alloc::vec::Vec<u8>);

impl From<NameId> for u64 {
    fn from(id: NameId) -> Self {
        id.value
    }
}

/// As `NameId` is constant for given input ID, which is only affected by
/// hashing function that's selected via compile features. It is safe to send a
/// it across different threads.
unsafe impl Send for NameId {}
/// `NameId` value is effectively a `u64`, with (optionally) some `'static`
/// metadata, which means that references to it can be safely shared across
/// threads.
unsafe impl Sync for NameId {}

impl core::fmt::Display for NameId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        #[cfg(all(debug_assertions, feature = "debug_name"))]
        {
            write!(f, "#{{{}}}", self.name)
        }
        #[cfg(not(all(debug_assertions, feature = "debug_name")))]
        {
            write!(f, "NameId({})", self.value)
        }
    }
}

impl core::fmt::Debug for NameId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        #[cfg(all(debug_assertions, feature = "debug_name"))]
        {
            write!(f, "NameId({})", self.name)
        }
        #[cfg(not(all(debug_assertions, feature = "debug_name")))]
        {
            write!(f, "NameId({})", self.value)
        }
    }
}
