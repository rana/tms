//! `vrn` module compresses integers with variable length encoding.
//! 
//! Variable length integer compression is also known as `VARINT`.
//! 
//! This is based on code from the Rust `integer-encoding` implementation.
//! 
//! See Rust implmentation https://docs.rs/integer-encoding/3.0.2/integer_encoding/index.html.
//! 
//! See Google documentation https://developers.google.com/protocol-buffers/docs/encoding.
//! 
//! See `VARINT` discussion in scholarly journal https://arxiv.org/abs/1401.6399.

/// `BYT_HDR_MSK` is a byte mask for the header of variable integer encoding. 
/// 
/// A header of 1 indicates another byte exist in the variable length encoding.
/// 
/// A header of 0 indicates that the current byte is the last byte of the variable length encoding.
const BYT_HDR_MSK: u8 = 0b1000_0000; // 0x80

/// `BYT_BDY_MSK` is a byte mask for the body of the variable integer encoding and decoding.
const BYT_BDY_MSK: u8 = 0b0111_1111;

/// `BIT_SHF_LEN` is the number of bits to shift during variable integer encoding and decoding.
const BIT_SHF_LEN: u8 = 7;

/// `usize_byt_len` returns the number of bytes to variable length encode the specfied usize value.
#[inline]
pub fn usize_byt_len(mut v: usize) -> usize {
  if v == 0 {
    return 1;
  }
  let mut len: usize = 0;
  while v > 0 {
    v >>= BIT_SHF_LEN;
    len += 1;
  }
  return len;
}
/// `usize_pck` encodes a usize to a [u8] and returns the number of bytes encoded.
#[inline]
pub fn usize_pck(mut v: usize, pck: &mut [u8]) -> usize {
  let mut p: usize = 0;
  while v >= 0x80 {
    pck[p] = BYT_HDR_MSK | (v as u8);
    v >>= BIT_SHF_LEN;
    p += 1;
  }
  pck[p] = v as u8;
  return p + 1;
}
/// `UsizeUnp` returns an unpacked usize value with the number of unpacked bytes.
#[derive(Debug)]
pub struct UsizeUnp {
  pub val: usize,
  pub len: usize,
}
/// `usize_unp` decodes a [u8] and returns a `UsizeUnp`.
#[inline]
pub fn usize_unp(pck: &[u8]) -> UsizeUnp {
  let mut r = UsizeUnp{
    val: 0,
    len: 0,
  };
  let mut shf: u8 = 0;
  for p in 0..pck.len() {  
    r.val |= ((pck[p] & BYT_BDY_MSK) as usize) << shf;
    r.len += 1;
    // Check if full integer is decoded
    if pck[p] & BYT_HDR_MSK == 0 {
      break;
    }
    shf += BIT_SHF_LEN;
  }
  return r;
}

/// `u32_byt_len` returns the number of bytes to variable length encode the specfied u32 value.
#[inline]
pub fn u32_byt_len(mut v: u32) -> usize {
  if v == 0 {
    return 1;
  }
  let mut len: usize = 0;
  while v > 0 {
    v >>= BIT_SHF_LEN;
    len += 1;
  }
  return len;
}
/// `u32_pck` encodes a u32 to a [u8] and returns the number of bytes encoded.
#[inline]
pub fn u32_pck(mut v: u32, pck: &mut [u8]) -> usize {
  let mut p: usize = 0;
  while v >= 0x80 {
    pck[p] = BYT_HDR_MSK | (v as u8);
    v >>= BIT_SHF_LEN;
    p += 1;
  }
  pck[p] = v as u8;
  return p + 1;
}
/// `U32Unp` returns an unpacked u32 value with the number of unpacked bytes.
#[derive(Debug)]
pub struct U32Unp {
  pub val: u32,
  pub len: usize,
}
/// `u32_unp` decodes a [u8] and returns a `U32Unp`.
#[inline]
pub fn u32_unp(pck: &[u8]) -> U32Unp {
  let mut r = U32Unp{
    val: 0,
    len: 0,
  };
  let mut shf: u8 = 0;
  for p in 0..pck.len() {  
    r.val |= ((pck[p] & BYT_BDY_MSK) as u32) << shf;
    r.len += 1;
    // Check if full integer is decoded
    if pck[p] & BYT_HDR_MSK == 0 {
      break;
    }
    shf += BIT_SHF_LEN;
  }
  return r;
}


/// `u32s_byt_len` returns the byte length of variable length encoded u32s.
#[inline]
pub fn u32s_byt_len(blk: &[u32]) -> usize {
  let mut len: usize = 0;
  for n in 0..blk.len() {
    if blk[n] == 0 {
      len += 1;
      continue;
    }
    let mut v = blk[n];
    while v > 0 {
      len += 1;
      v >>= BIT_SHF_LEN;
    }
  }
  
  return len;
}

/// `u32s_pck` compresses u32s to variable length encoded bytes.
/// 
/// `dst` is expected to be large enough.
#[inline]
pub fn u32s_pck(src: &[u32], dst: &mut [u8]) {
  let mut d: usize = 0;
  for s in 0..src.len() {
    let mut v = src[s];
    while v >= 0x80 {
      dst[d] = BYT_HDR_MSK | (v as u8);
      v >>= BIT_SHF_LEN;
      d += 1;
    }
    dst[d] = v as u8;
    d += 1;
  }
}

/// `u32s_unp` decompresses u32s from variable length encoded bytes.
/// 
/// `dst` is expected to be zeroed out and large enough.
#[inline]
pub fn  u32s_unp(src: &[u8], dst: &mut [u32]) {
  let mut shf: u8 = 0;
  let mut d: usize = 0;
  for s in 0..src.len() {
    dst[d] |= ((src[s] & BYT_BDY_MSK) as u32) << shf;
    shf += BIT_SHF_LEN;
    // Check if full integer is decoded
    if src[s] & BYT_HDR_MSK == 0 {
      shf = 0;
      d += 1;
    }
  }
}

#[cfg(test)]
mod tst {
  use super::*;
  use usize;

  #[test]
  fn usize_byt_len_() {
    assert_eq!(1, usize_byt_len(0));
    assert_eq!(1, usize_byt_len(1));
    assert_eq!(1, usize_byt_len(usize::pow(2, 7)-1));
    assert_eq!(2, usize_byt_len(usize::pow(2, 7)));
    assert_eq!(2, usize_byt_len(usize::pow(2, 2*7)-1));
    assert_eq!(3, usize_byt_len(usize::pow(2, 2*7)));
    assert_eq!(3, usize_byt_len(usize::pow(2, 3*7)-1));
    assert_eq!(4, usize_byt_len(usize::pow(2, 3*7)));
    assert_eq!(4, usize_byt_len(usize::pow(2, 4*7)-1));
    assert_eq!(5, usize_byt_len(usize::pow(2, 4*7)));
    assert_eq!(5, usize_byt_len(usize::pow(2, 5*7)-1));
    assert_eq!(6, usize_byt_len(usize::pow(2, 5*7)));
    assert_eq!(6, usize_byt_len(usize::pow(2, 6*7)-1));
    assert_eq!(7, usize_byt_len(usize::pow(2, 6*7)));
    assert_eq!(7, usize_byt_len(usize::pow(2, 7*7)-1));
    assert_eq!(8, usize_byt_len(usize::pow(2, 7*7)));
    assert_eq!(8, usize_byt_len(usize::pow(2, 8*7)-1));
  }
  #[test]
  fn usize_pck_unp() {
    let mut pck = vec![0u8; 8];
    let mut val: usize = 0;
    let mut len: usize = 1;
    assert_eq!(len, usize_pck(val, &mut pck));
    let r = usize_unp(&pck);
    assert_eq!(val, r.val);
    assert_eq!(len, r.len);

    pck = vec![0u8; 8];
    val = 1;
    len = 1;
    assert_eq!(len, usize_pck(val, &mut pck));
    let r = usize_unp(&pck);
    assert_eq!(val, r.val);
    assert_eq!(len, r.len);
    
    pck = vec![0u8; 8];
    val = usize::pow(2, 7)-1;
    len = 1;
    assert_eq!(len, usize_pck(val, &mut pck));
    let r = usize_unp(&pck);
    assert_eq!(val, r.val);
    assert_eq!(len, r.len);

    pck = vec![0u8; 8];
    val = usize::pow(2, 7);
    len = 2;
    assert_eq!(len, usize_pck(val, &mut pck));
    let r = usize_unp(&pck);
    assert_eq!(val, r.val);
    assert_eq!(len, r.len);

    pck = vec![0u8; 8];
    val = usize::pow(2, 2*7)-1;
    len = 2;
    assert_eq!(len, usize_pck(val, &mut pck));
    let r = usize_unp(&pck);
    assert_eq!(val, r.val);
    assert_eq!(len, r.len);

    pck = vec![0u8; 8];
    val = usize::pow(2, 2*7);
    len = 3;
    assert_eq!(len, usize_pck(val, &mut pck));
    let r = usize_unp(&pck);
    assert_eq!(val, r.val);
    assert_eq!(len, r.len);

    pck = vec![0u8; 8];
    val = usize::pow(2, 3*7)-1;
    len = 3;
    assert_eq!(len, usize_pck(val, &mut pck));
    let r = usize_unp(&pck);
    assert_eq!(val, r.val);
    assert_eq!(len, r.len);

    pck = vec![0u8; 8];
    val = usize::pow(2, 3*7);
    len = 4;
    assert_eq!(len, usize_pck(val, &mut pck));
    let r = usize_unp(&pck);
    assert_eq!(val, r.val);
    assert_eq!(len, r.len);

    pck = vec![0u8; 8];
    val = usize::pow(2, 4*7)-1;
    len = 4;
    assert_eq!(len, usize_pck(val, &mut pck));
    let r = usize_unp(&pck);
    assert_eq!(val, r.val);
    assert_eq!(len, r.len);

    pck = vec![0u8; 8];
    val = usize::pow(2, 4*7);
    len = 5;
    assert_eq!(len, usize_pck(val, &mut pck));
    let r = usize_unp(&pck);
    assert_eq!(val, r.val);
    assert_eq!(len, r.len);

    pck = vec![0u8; 8];
    val = usize::pow(2, 5*7)-1;
    len = 5;
    assert_eq!(len, usize_pck(val, &mut pck));
    let r = usize_unp(&pck);
    assert_eq!(val, r.val);
    assert_eq!(len, r.len);

    pck = vec![0u8; 8];
    val = usize::pow(2, 5*7);
    len = 6;
    assert_eq!(len, usize_pck(val, &mut pck));
    let r = usize_unp(&pck);
    assert_eq!(val, r.val);
    assert_eq!(len, r.len);

    pck = vec![0u8; 8];
    val = usize::pow(2, 6*7)-1;
    len = 6;
    assert_eq!(len, usize_pck(val, &mut pck));
    let r = usize_unp(&pck);
    assert_eq!(val, r.val);
    assert_eq!(len, r.len);

    pck = vec![0u8; 8];
    val = usize::pow(2, 6*7);
    len = 7;
    assert_eq!(len, usize_pck(val, &mut pck));
    let r = usize_unp(&pck);
    assert_eq!(val, r.val);
    assert_eq!(len, r.len);

    pck = vec![0u8; 8];
    val = usize::pow(2, 7*7)-1;
    len = 7;
    assert_eq!(len, usize_pck(val, &mut pck));
    let r = usize_unp(&pck);
    assert_eq!(val, r.val);
    assert_eq!(len, r.len);

    pck = vec![0u8; 8];
    val = usize::pow(2, 7*7);
    len = 8;
    assert_eq!(len, usize_pck(val, &mut pck));
    let r = usize_unp(&pck);
    assert_eq!(val, r.val);
    assert_eq!(len, r.len);

    pck = vec![0u8; 8];
    val = usize::pow(2, 8*7)-1;
    len = 8;
    assert_eq!(len, usize_pck(val, &mut pck));
    let r = usize_unp(&pck);
    assert_eq!(val, r.val);
    assert_eq!(len, r.len);
  }

  #[test]
  fn u32_byt_len_() {
    assert_eq!(1, u32_byt_len(0));
    assert_eq!(1, u32_byt_len(1));
    assert_eq!(1, u32_byt_len(u32::pow(2, 7)-1));
    assert_eq!(2, u32_byt_len(u32::pow(2, 7)));
    assert_eq!(2, u32_byt_len(u32::pow(2, 2*7)-1));
    assert_eq!(3, u32_byt_len(u32::pow(2, 2*7)));
    assert_eq!(3, u32_byt_len(u32::pow(2, 3*7)-1));
    assert_eq!(4, u32_byt_len(u32::pow(2, 3*7)));
    assert_eq!(4, u32_byt_len(u32::pow(2, 4*7)-1));
  }
  #[test]
  fn u32_pck_unp() {
    let mut pck = vec![0u8; 8];
    let mut val: u32 = 0;
    let mut len: usize = 1;
    assert_eq!(len, u32_pck(val, &mut pck));
    let r = u32_unp(&pck);
    assert_eq!(val, r.val);
    assert_eq!(len, r.len);

    pck = vec![0u8; 8];
    val = 1;
    len = 1;
    assert_eq!(len, u32_pck(val, &mut pck));
    let r = u32_unp(&pck);
    assert_eq!(val, r.val);
    assert_eq!(len, r.len);
    
    pck = vec![0u8; 8];
    val = u32::pow(2, 7)-1;
    len = 1;
    assert_eq!(len, u32_pck(val, &mut pck));
    let r = u32_unp(&pck);
    assert_eq!(val, r.val);
    assert_eq!(len, r.len);

    pck = vec![0u8; 8];
    val = u32::pow(2, 7);
    len = 2;
    assert_eq!(len, u32_pck(val, &mut pck));
    let r = u32_unp(&pck);
    assert_eq!(val, r.val);
    assert_eq!(len, r.len);

    pck = vec![0u8; 8];
    val = u32::pow(2, 2*7)-1;
    len = 2;
    assert_eq!(len, u32_pck(val, &mut pck));
    let r = u32_unp(&pck);
    assert_eq!(val, r.val);
    assert_eq!(len, r.len);

    pck = vec![0u8; 8];
    val = u32::pow(2, 2*7);
    len = 3;
    assert_eq!(len, u32_pck(val, &mut pck));
    let r = u32_unp(&pck);
    assert_eq!(val, r.val);
    assert_eq!(len, r.len);

    pck = vec![0u8; 8];
    val = u32::pow(2, 3*7)-1;
    len = 3;
    assert_eq!(len, u32_pck(val, &mut pck));
    let r = u32_unp(&pck);
    assert_eq!(val, r.val);
    assert_eq!(len, r.len);

    pck = vec![0u8; 8];
    val = u32::pow(2, 3*7);
    len = 4;
    assert_eq!(len, u32_pck(val, &mut pck));
    let r = u32_unp(&pck);
    assert_eq!(val, r.val);
    assert_eq!(len, r.len);

    pck = vec![0u8; 8];
    val = u32::pow(2, 4*7)-1;
    len = 4;
    assert_eq!(len, u32_pck(val, &mut pck));
    let r = u32_unp(&pck);
    assert_eq!(val, r.val);
    assert_eq!(len, r.len);
  }

  #[test]
  fn u32s_byt_len_() {
    assert_eq!(0, u32s_byt_len(vec![0; 0].as_slice()));
    assert_eq!(1, u32s_byt_len(vec![0].as_slice()));
    assert_eq!(2, u32s_byt_len(vec![0, 1].as_slice()));
    assert_eq!(11, u32s_byt_len(vec![0, 1, 128, 16384, 2097152].as_slice()));
  }

  #[test]
  fn u32s_pck_unp_empty() {
    let unp_src = vec![];
    // Clone unp_src which will be overwritten
    let unp_exp = unp_src.clone();
    let mut pck_act = vec![0; u32s_byt_len(&unp_src)];
    u32s_pck(&unp_src, &mut pck_act);
    let mut unp_act = vec![0; unp_src.len()];
    u32s_unp(&pck_act, &mut unp_act);
    assert_eq!(unp_exp, unp_act);
  }

  #[test]
  fn u32s_pck_unp_single_0() {
    let unp_src = vec![0];
    // Clone unp_src which will be overwritten
    let unp_exp = unp_src.clone();
    let mut pck_act = vec![0; u32s_byt_len(&unp_src)];
    u32s_pck(& unp_src, &mut pck_act);
    let mut unp_act = vec![0; unp_src.len()];
     u32s_unp(&pck_act, &mut unp_act);
    assert_eq!(unp_exp, unp_act);
  }

  #[test]
  fn u32s_pck_unp_single_1() {
    let unp_src = vec![1];
    // Clone unp_src which will be overwritten
    let unp_exp = unp_src.clone();
    let mut pck_act = vec![0; u32s_byt_len(&unp_src)];
    u32s_pck(& unp_src, &mut pck_act);
    let mut unp_act = vec![0; unp_src.len()];
     u32s_unp(&pck_act, &mut unp_act);
    assert_eq!(unp_exp, unp_act);
  }

  #[test]
  fn u32s_pck_unp_single_128() {
    let unp_src = vec![128];
    // Clone unp_src which will be overwritten
    let unp_exp = unp_src.clone();
    let mut pck_act = vec![0; u32s_byt_len(&unp_src)];
    u32s_pck(& unp_src, &mut pck_act);
    let mut unp_act = vec![0; unp_src.len()];
     u32s_unp(&pck_act, &mut unp_act);
    assert_eq!(unp_exp, unp_act);
  }

  #[test]
  fn u32s_pck_unp_single_16384() {
    let unp_src = vec![16384];
    // Clone unp_src which will be overwritten
    let unp_exp = unp_src.clone();
    let mut pck_act = vec![0; u32s_byt_len(&unp_src)];
    u32s_pck(& unp_src, &mut pck_act);
    let mut unp_act = vec![0; unp_src.len()];
     u32s_unp(&pck_act, &mut unp_act);
    assert_eq!(unp_exp, unp_act);
  }

  #[test]
  fn u32s_pck_unp_single_2097152() {
    let unp_src = vec![2097152];
    // Clone unp_src which will be overwritten
    let unp_exp = unp_src.clone();
    let mut pck_act = vec![0; u32s_byt_len(&unp_src)];
    u32s_pck(& unp_src, &mut pck_act);
    let mut unp_act = vec![0; unp_src.len()];
     u32s_unp(&pck_act, &mut unp_act);
    assert_eq!(unp_exp, unp_act);
  }

  #[test]
  fn u32s_pck_unp_multiple() {
    let unp_src = vec![0, 1, 128, 16384, 2097152];
    // Clone unp_src which will be overwritten
    let unp_exp = unp_src.clone();
    let mut pck_act = vec![0; u32s_byt_len(&unp_src)];
    u32s_pck(& unp_src, &mut pck_act);
    let mut unp_act = vec![0; unp_src.len()];
     u32s_unp(&pck_act, &mut unp_act);
    assert_eq!(unp_exp, unp_act);
  }
}