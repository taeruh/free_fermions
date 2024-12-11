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
    // pub type int = u32;
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

    pub fn to_float(x: int) -> f64 {
        // int=u32::MAX = 4294967295 < 1.7976931348623157e308 = f64::MAX
        x as f64
    }
}

/// Don't use this function; it's just an unsafe marker.
/// # Safety
/// None
#[inline(always)]
pub(crate) unsafe fn unsafe_marker() {}

#[macro_export]
macro_rules! debug_unreachable_unchecked {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        {
            $crate::unsafe_marker();
            unreachable!($($arg)*);
        }
        #[cfg(not(debug_assertions))]
        std::hint::unreachable_unchecked();
    };
}

pub mod enumerate_offset;
pub mod graph;
// pub mod old_hamiltonian;
pub mod hamiltonian;
pub mod mat_mul;
pub mod matrix;
// pub mod playing_around;
pub mod run;
pub mod rand_helper;
