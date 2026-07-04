use rand::{Error, RngCore};
use rp_pico::pac::ROSC;

pub struct RoscRng;

impl RoscRng {
    #[inline]
    fn random_byte() -> u8 {
        let mut b = 0_u8;
        for i in 0..8 {
            let bits = unsafe { (*ROSC::PTR).randombit.read().bits() } as u8;
            b |= (bits & 0b1) << i;
        }
        b
    }
}

impl RngCore for RoscRng {
    fn next_u32(&mut self) -> u32 {
        let mut u = [0_u8; 4];
        self.fill_bytes(&mut u);
        u32::from_le_bytes(u)
    }

    fn next_u64(&mut self) -> u64 {
        let mut u = [0_u8; 8];
        self.fill_bytes(&mut u);
        u64::from_le_bytes(u)
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        for x in dest.iter_mut() {
            *x = Self::random_byte();
        }
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}
