use digest::generic_array::GenericArray;
use digest::typenum::Unsigned;
use digest::{Digest, FixedOutput, OutputSizeUser};

pub fn fixed_output_hash<H>(data: &[u8], iter_num: u64) -> [u8; H::OutputSize::USIZE]
where
    H: Digest + FixedOutput + OutputSizeUser,
    [(); H::OutputSize::USIZE]:,
    GenericArray<u8, H::OutputSize>: From<[u8; H::OutputSize::USIZE]>,
    GenericArray<u8, H::OutputSize>: Into<[u8; H::OutputSize::USIZE]>,
{
    // initial hashing
    let mut hash = GenericArray::<u8, H::OutputSize>::from([0_u8; H::OutputSize::USIZE]);
    let mut hasher = H::new();
    Digest::update(&mut hasher, data);
    FixedOutput::finalize_into(hasher, &mut hash);

    // last iterations
    for _ in 1..iter_num {
        let mut hasher = H::new();
        Digest::update(&mut hasher, &*hash);
        FixedOutput::finalize_into(hasher, &mut hash);
    }

    hash.into()
}
