use name_id::{id, NameId};

// any ident token will be treated as a string
const IDENT_SINGLE: NameId = id!(some_id_ident);
// multiple tokens will be concatenated into a string with ' ' delimiter
const IDENT_SEQUENCE: NameId = id!(can even be 6 or more);
// string representations will be used in case of literals
const STRING_ID: NameId = id!("id macro supports string values");
// so for numbers, their string representation will be hashed
const NUMBER_ID: NameId = id!(256);
// and any valid utf-8 character can be used
const SPECIAL_ID: NameId = id!("!%$#");

#[allow(unused_assignments)]
fn main() {
    // NameId can be checked for equality against other NameIds
    assert_eq!(IDENT_SINGLE, NameId::new("some_id_ident"));
    // they can also be checked against any AsRef<str>, which will be
    // automatically hashed for comparison using the same hashing algorithm the
    // crate uses
    assert_eq!(IDENT_SEQUENCE, "can even be 6 or more");

    // hash values can be accessed via a const function
    #[cfg(feature = "ahash")]
    assert_eq!(STRING_ID.value(), 10398550419565578837);

    let are_equal = const {
        let mut const_variable = STRING_ID;
        const_variable = NUMBER_ID;

        // there are also const_eq and const_cmp utility functions for checking
        // whether IDs are equal at compile time:
        NUMBER_ID.const_eq(&const_variable)
    };

    if are_equal && SPECIAL_ID == "!%$#" {
        println!("All checks passed.");
    }
}
