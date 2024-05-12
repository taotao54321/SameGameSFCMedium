//! 各種ビット演算。

use std::arch::x86_64::_pext_u32;

/// 最下位ビットを分離する。たとえば `0b110100` に対しては `0b000100` を返す。
/// 引数が 0 の場合、0 を返す。
pub const fn u32_blsi(x: u32) -> u32 {
    x & x.wrapping_neg()
}

/// 1 のビットのインデックスを昇順で列挙する。
pub fn u32_one_indexs(x: u32) -> U32OneIndexs {
    U32OneIndexs(x)
}

/// PEXT 命令。
pub fn u32_pext(x: u32, mask: u32) -> u32 {
    unsafe { _pext_u32(x, mask) }
}

#[repr(transparent)]
#[derive(Clone)]
pub struct U32OneIndexs(u32);

impl Iterator for U32OneIndexs {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == 0 {
            return None;
        }

        let i = self.0.trailing_zeros();
        self.0 &= !(1 << i);

        Some(i)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.0.count_ones() as usize;

        (n, Some(n))
    }
}

impl ExactSizeIterator for U32OneIndexs {}

impl std::iter::FusedIterator for U32OneIndexs {}
