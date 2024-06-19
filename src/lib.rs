/*!
# ***IMPORTANT:***
**This library only safely works on 32bit, or higher, platforms (for performance reasons)!
Use [fix_int::ensure_at_least_32bit_platform], *before calling any other code in this
library,* to check that you are on a 32bit platform!**

For an application, copy-paste the following into your main function:
```rust
// because this library unsafely relies on that
free_fermions::fix_int::ensure_at_least_32bit_platform();
```
And for test code, add the following to your according test module:
```rust,ignore
#[cfg(test)]
#[test]
fn test_ensure_at_least_32bit_platform() {
    free_fermions::fix_int::ensure_at_least_32bit_platform();
}
```
*/

#![deny(unsafe_op_in_unsafe_fn)]

macro_rules! non_semantic_default {
    () => {
        "Note that semantically, this impl makes not much sense. It is rather useful for \
         initialization."
    };
}

pub mod fix_int {
    #[allow(non_camel_case_types)]
    /// A fixed (unsized!) integer type.
    ///
    /// This type is generally used in this library to represent node/vertex labels in
    /// graphs (and related stuff).
    pub type int = u32;

    /// Run this to check that we are on a 32bit platform.
    pub fn ensure_at_least_32bit_platform() {
        // make sure that we are on a 32bit platform
        let usize_size = std::mem::size_of::<usize>();
        assert!(
            usize_size >= 4,
            "This library is only for >=32bit platforms, but the usize size is {} bytes.",
            usize_size
        );
    }

    #[cfg(test)]
    #[test]
    fn test_ensure_at_least_32bit_platform() {
        ensure_at_least_32bit_platform();
    }

    /// Use the [int] type to enumerate.
    macro_rules! enumerate {
        ($iter:expr) => {
            // always make sure that we actually use the int type here
            (0u32..).zip($iter)
        };
    }
    pub(crate) use enumerate;
}

pub mod algorithms;
pub mod enumerate_offset;
pub mod graph;
pub mod hamiltonian;
pub mod mat_mul;
pub mod matrix;
pub mod playing_around;
pub mod run;

// pub mod mat_mul;
