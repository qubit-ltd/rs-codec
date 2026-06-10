use core::num::NonZeroUsize;

pub(crate) fn nz(value: usize) -> NonZeroUsize {
    NonZeroUsize::new(value).expect("test additional count must be non-zero")
}
