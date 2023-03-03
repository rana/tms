
use rand::prelude::*;
use rand::distributions::{Uniform};
use std::u32;
use std::cmp::max;
// for consts
use super::*; 

pub mod goog;

/// `gen_blk` generates a sorted block of u32 values.
/// 
/// The maximum difference between each value is the specified bit-length.
/// 
/// At lease one difference has the maximum specified bit-length.
/// 
/// The starting value is zero.
pub fn gen_blk(blk_len: usize, bit_len: u8) -> Vec<u32> {
  assert!(blk_len >= MIN_ELM_PER_BLK);
  assert!(blk_len % ELM_PER_SMD == 0);
  assert!(bit_len <= 32u8);
  

  // Initialize a vector filled with zeros
  let mut blk = vec![0u32; blk_len];

  // Return all zeros when bit-length is zero
  if bit_len == 0u8 {
    return blk;
  }

  // Setup random number generation
  // The random number is the difference between generated values
  let max_excl = if bit_len < 32u8 {
    1u32 << bit_len
  } else {
    u32::MAX
  };
  let dist = Uniform::new(0, max_excl);
  let mut rng = thread_rng();

  // Calculate the maximum difference based on the bit-length
  let dlt_max = if bit_len < 32u8 {
    (1u32 << bit_len) - 1
  } else {
    u32::MAX
  };

  // Calculations are done on a per SIMD vector basis
  // Initialize the first SIMD vector values with zero
  // Start the second SIMD vector with all maximum differences from the first value
  // This ensures that the block has at least one difference with the specified bit-length
  for idx in ELM_PER_SMD..(ELM_PER_SMD*2) {
    blk[idx] = dlt_max;
  }

  // Determine the number of SIMD vectors
  let smd_len: usize = blk_len / ELM_PER_SMD;
  
  // Increment by SIMD vectors
  // The bit-length calculation is done on with SIMD vector lengths
  // Produce ascending values starting from zero
  // The maximum difference between values has a bit-length of `bit_len`
  // Once u32::MAX is reached reached, repeat u32::MAX to produce differences of zero
  for smd_idx in 2..smd_len {

    let mut rnds: Vec<u32> = (0..ELM_PER_SMD).map(|_|dist.sample(&mut rng)).collect();
    rnds.sort();
    
    for lne_idx in 0..ELM_PER_SMD {

      // Calculate the previous block index
      let prv_blk_idx = ((smd_idx-1) * ELM_PER_SMD) + lne_idx;

      // Calculate the current block index
      let cur_blk_idx = (smd_idx * ELM_PER_SMD) + lne_idx;

      // Calculate current value based neighbor element to ensure sort order
      // Use `saturating_add()` to repeat u32::MAX at the end, if necessary
      blk[cur_blk_idx] = blk[cur_blk_idx-1].saturating_add(rnds[lne_idx]);

      // Determine current difference from previous SIMD vector
      let dlt = blk[cur_blk_idx] - blk[prv_blk_idx];

      // Ensure the current value does not produce a SIMD difference larger than bit-length
      if dlt > dlt_max {
        blk[cur_blk_idx] = blk[prv_blk_idx] + dlt_max;
      }
    }
  }
  
  return blk;
}
