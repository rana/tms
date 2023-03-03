//! `mcr` module provides procedural macros.
//!
//! Procedural macros must exist in their own crate.
//!
//! For an example reference see
//! https://github.com/dtolnay/syn/blob/master/examples/lazy-static/lazy-static/src/lib.rs

#![cfg_attr(debug_assertions, allow(dead_code, unused_imports, unused_variables))]
#![feature(proc_macro_diagnostic)]
#![feature(extend_one)]

use proc_macro::TokenStream;
use proc_macro2::{Group, Span, TokenTree};
use quote::quote;
// use std::error::Error;
use syn::parse::{Parse, ParseStream, Result};
use syn::{parse_macro_input, parse_quote, LitInt};

/// `BIT_PER_BYT` is the number of bits per byte.
const BIT_PER_BYT: usize = 8;
/// `BIT_PER_ELM` is the number of bits per 32-bit element.
const BIT_PER_ELM: usize = 32;
/// `BYT_PER_ELM` is the number of bytes per 32-bit element.
const BYT_PER_ELM: usize = 4;
/// `BIT_PER_LNE` is the number of bits per SIMD lane.
/// 
/// This is the same as `BIT_PER_ELM`, and can be convient to think in terms of SIMD lanes.
const BIT_PER_LNE: usize = 32;
/// `BIT_PER_SMD` is the number of bits per SIMD vector.
const BIT_PER_SMD: usize = 256;
/// `BYT_PER_SMD` is the number of bytes per SIMD vector.
const BYT_PER_SMD: usize = 32;
/// `ELM_PER_SMD` is the number of 32-bit elements per SIMD vector.
const ELM_PER_SMD: usize = 8;
/// `MIN_ELM_PER_BLK` is the inclusive minimum number of 32-bit elements per block.
/// This is two SIMD vectors.
const MIN_ELM_PER_BLK: usize = 16;


/// `blk_itr` returns a block iterator of shift operations for packing or unpacking.
fn blk_itr(elm_bit_len: usize, elm_per_blk: usize) -> BlkItr {
  BlkItr{
    // is_pck: is_pck,
    bit_per_lne: BIT_PER_LNE,
    elm_bit_len,
    elm_per_blk,
    // TODO: WHAT IF lne_bit_lim IS BIT_PER_LANE? IE, NO COMPRESSION...
    lne_bit_lim: BIT_PER_LNE - elm_bit_len + 1,
    // Calculate the total number of bits to be packed for this block
    blk_bit_len: elm_bit_len * elm_per_blk,
    // Calculate the number of bits packed so far.
    // blk_bit_sum: 0,
    prv: Itm{
      lne_bit_len: 0,
      lne_bit_sum: 0,
      blk_bit_sum: 0,
      shf_len: 0,
      shf_dir: Dir::Zro,
    }
  }
}
/// `BlkItr` provides block iteration of shift operations for packing or unpacking.
#[derive(Debug)]
struct BlkItr {
  bit_per_lne: usize,
  elm_bit_len: usize,
  elm_per_blk: usize,
  lne_bit_lim: usize,
  blk_bit_len: usize,
  prv: Itm,
}
#[derive(Debug, Clone, Copy)]
enum Dir {
  Zro,
  Bck,
  Fwd,
  FwdPrt,
}
#[derive(Debug)]
enum Itr {
  /// `Fst` is the first iteration.
  Fst,
  /// `Mdl` is a middle iteration.
  Mdl { itm: Itm },
  /// `Lst` is the last iteration.
  Lst { itm: Itm },
}
#[derive(Debug, Clone, Copy)]
struct Itm {
  /// `shf_dir` is the direction of a bit-sift operation.
  shf_dir: Dir,
  /// `shf_len` is the number of the bits to shift.
  shf_len: usize,
  /// `lne_bit_len` is the number of bits shifted within the SIMD lane during the current iteration.
  lne_bit_len: usize,
  /// `lne_bit_sum` is the cummulative number of bits shifted within the SIMD lane during the current iteration.
  lne_bit_sum: usize,
  /// `blk_bit_sum` is the cummulative number of bits shifted within the block during the current iteration.
  blk_bit_sum: usize,
}
impl Iterator for BlkItr {
  type Item = Itr;
  fn next(&mut self) -> Option<Itr> {

    // After Last iteration. Stop iterating when all bits are packed or unpacked
    if self.prv.blk_bit_sum == self.blk_bit_len {
      return None;
    }

    // Determine the current lane bits packed or unpacked, and lane bit sum, etc
    let cur = if self.prv.blk_bit_sum == 0 {
      // First iteration
      let lne_bit_len = self.elm_bit_len;
      Itm{
        shf_dir: Dir::Zro,
        shf_len: 0,
        lne_bit_len: lne_bit_len,
        lne_bit_sum: lne_bit_len,
        blk_bit_sum: lne_bit_len * ELM_PER_SMD,
      }
    } else if self.prv.lne_bit_sum < self.lne_bit_lim {
      // Previously 14 bits packed for op, 16 bits packed cummulatively to SIMD lane
      // Previously 14 bits packed for op, 18 bits packed cummulatively to SIMD lane
      let lne_bit_len = self.elm_bit_len;
      Itm{
        shf_dir: Dir::Fwd,
        shf_len: self.prv.lne_bit_sum,
        lne_bit_len: lne_bit_len,
        lne_bit_sum: self.prv.lne_bit_sum + self.elm_bit_len,
        blk_bit_sum: self.prv.blk_bit_sum + (lne_bit_len * ELM_PER_SMD),
      }
    } else if self.prv.lne_bit_sum < self.bit_per_lne {
      // Previously 14 bits packed for op, 30 bits packed cummulatively to SIMD lane
      let lne_bit_len = self.bit_per_lne - self.prv.lne_bit_sum;
      Itm{
        shf_dir: Dir::FwdPrt,
        shf_len: self.prv.lne_bit_sum,
        lne_bit_len: lne_bit_len,
        lne_bit_sum: self.bit_per_lne,
        blk_bit_sum: self.prv.blk_bit_sum + (lne_bit_len * ELM_PER_SMD),
      }
    } else {
      // Previously 32 bits packed cummulatively to SIMD lane
      if self.prv.lne_bit_len == self.elm_bit_len  {
        // Previously 14 bits packed for op, 32 bits packed cummulatively to SIMD lane
        let lne_bit_len = self.elm_bit_len;
        Itm{
          shf_dir: Dir::Zro,
          shf_len: 0,
          lne_bit_len: lne_bit_len,
          lne_bit_sum: lne_bit_len,
          blk_bit_sum: self.prv.blk_bit_sum + (lne_bit_len * ELM_PER_SMD),
        }
      } else {
        // Previously 10 bits packed for op, 32 bits packed cummulatively to SIMD lane
        let lne_bit_len = self.elm_bit_len - self.prv.lne_bit_len;
        Itm{
          shf_dir: Dir::Bck,
          shf_len: self.prv.lne_bit_len,
          lne_bit_len: lne_bit_len,
          lne_bit_sum: lne_bit_len,
          blk_bit_sum: self.prv.blk_bit_sum + (lne_bit_len * ELM_PER_SMD),
        }
      }
    };

    // Determine iteration order
    let result = if self.prv.blk_bit_sum == 0 {
      Itr::Fst
    } else if cur.blk_bit_sum != self.blk_bit_len {
      Itr::Mdl{itm: cur}
    } else {
      Itr::Lst{itm: cur}
    };

    // Set previous item for next iteration
    self.prv = cur;

    return Some(result);
  }
}

// u32_blk_pck creates a u32 pack method as a TokenStream.
fn u32_blk_pck(elm_per_blk: usize, smd_per_blk: usize) -> proc_macro2::TokenStream {
  let pck_name = proc_macro2::Ident::new(&format!("u32x{}_pck", elm_per_blk), Span::call_site());

  // Define a match tree
  let mut match_tree: Vec<TokenTree> = quote! {
    match elm_bit_len {}
  }
  .into_iter()
  .collect();

  if let TokenTree::Group(g_tree) = match_tree.pop().unwrap() {
    let mut gs_tree = g_tree.stream();

    // Arm 0
    // No packing or copying occurs at a bit-length of 0
    gs_tree.extend_one(quote! {
      0u8 => {},
    });

    // Cycle through [1, 32) bit-lengths
    for elm_bit_len in 1..32u8 {
      
      // Define a match arm for current bit-length
      let mut match_arm: Vec<TokenTree> = quote! {
        #elm_bit_len => {},
      }
      .into_iter()
      .collect();
     
      // Unroll the loop calculation with shift sizes at each iteration.
      // Expect that the specified unpacked block is exactly `elm_per_blk` size.
      if let TokenTree::Group(g) = &match_arm[3] {
        let mut gs = g.stream();
        // --- arm: start

        // pck_ptr_off & unp_ptr_off required to be `usize`.
        // Each increment will offset by a single SIMD vector.
        let mut pck_ptr_off: usize = 0;
        let mut unp_ptr_off: usize = 1;

        // Iterate through shift operations for packing
        for cur in blk_itr(elm_bit_len as usize, elm_per_blk) {
          match cur {
            Itr::Fst => {
              // No shift for 1st SIMD lane
              // 1st SIMD vector starts with all first u32s filled
              gs.extend_one(quote! {
                let mut prv = set1(fst as i32);
                let mut cur = load(unp_ptr);
                let mut smd_pck = sub(cur, prv);
                prv = cur;
              });
            },
            Itr::Mdl{ itm } => {
              let shf_lit = proc_macro2::Literal::i32_suffixed(itm.shf_len as i32);
              match itm.shf_dir {
                Dir::Zro => {
                  // No shift
                  gs.extend_one(quote! {
                    cur = load(unp_ptr.add(#unp_ptr_off));
                    smd_pck = sub(cur, prv);
                    prv = cur;
                  });
                  unp_ptr_off += 1;
                },
                Dir::Bck => {
                  // Partial right shift
                  gs.extend_one(quote! {
                    smd_pck = rht(dlt, #shf_lit);
                  });
                },
                Dir::Fwd => {
                  // Full left shift
                  gs.extend_one(quote! {
                    cur = load(unp_ptr.add(#unp_ptr_off));
                    smd_pck = or(smd_pck, lft(sub(cur, prv), #shf_lit));
                    prv = cur;
                  });
                  unp_ptr_off += 1;
                },
                Dir::FwdPrt => {
                  // Partial left shift
                  gs.extend_one(quote! {
                    cur = load(unp_ptr.add(#unp_ptr_off));
                    let dlt = sub(cur, prv);
                    smd_pck = or(smd_pck, lft(dlt, #shf_lit));
                    prv = cur;
                  });
                  unp_ptr_off += 1;
                },
              }

              // Write fully packed SIMD vector
              if itm.lne_bit_sum == BIT_PER_LNE {
                gs.extend_one(quote! {
                  store(pck_ptr.add(#pck_ptr_off), smd_pck);
                });
                pck_ptr_off += 1;
              }
            },
            Itr::Lst{ itm } => {
              // Write last packed SIMD vector. The vector may be partially packed or fully packed
              let shf_lit = proc_macro2::Literal::i32_suffixed(itm.shf_len as i32);
              match itm.shf_dir {
                Dir::Zro => {
                  // No shift
                  gs.extend_one(quote! {
                    store(
                      pck_ptr.add(#pck_ptr_off),
                      sub(load(unp_ptr.add(#unp_ptr_off)), prv),
                    );
                  });
                },
                Dir::Bck => {
                  // Partial right shift
                  gs.extend_one(quote! {
                    store(
                      pck_ptr.add(#pck_ptr_off),
                      rht(dlt, #shf_lit),
                    );
                  });
                },
                Dir::Fwd | Dir::FwdPrt => {
                  // Full left shift or Partial left shift
                  gs.extend_one(quote! {
                    store(
                      pck_ptr.add(#pck_ptr_off),
                      or(smd_pck, lft(sub(load(unp_ptr.add(#unp_ptr_off)), prv), #shf_lit)),
                    );
                  });
                },
              }
            },
          }
        }

        // --- arm: end
        // Push unrolled loop into arm definition
        match_arm[3] = Group::new(g.delimiter(), gs.into()).into();
      }

      // Push the match arm onto the match tree
      gs_tree.extend(match_arm);
    }

    // Arm 32
    // No packing occurs at a bit-length of 32
    // Copy raw bytes
    gs_tree.extend_one(quote! {
      32u8 => {
        // See https://doc.rust-lang.org/src/core/slice/mod.rs.html#3065
        ptr::copy_nonoverlapping(
          unp.as_ptr() as *const u8, 
          pck.as_mut_ptr(), 
          pck.len());
      },
      _ => panic!("unsupported bit-length {}", elm_bit_len)
    });

    // Push new elements into tree
    match_tree.push(Group::new(g_tree.delimiter(), gs_tree.into()).into());
  }
  let match_arms: proc_macro2::TokenStream = match_tree.into_iter().collect();  

  // Create the pack method
  return quote! {
    pub unsafe fn #pck_name(elm_bit_len: u8, fst: u32, unp: &[u32], pck: &mut [u8]) {
      let unp_ptr = unp.as_ptr() as *const m256;
      let pck_ptr = pck.as_mut_ptr() as *mut m256;

      #match_arms
    }
  };
}

// u32_blk_unp creates a u32 unpack method as a TokenStream.
fn u32_blk_unp(elm_per_blk: usize, smd_per_blk: usize) -> proc_macro2::TokenStream {
  let unp_name = proc_macro2::Ident::new(&format!("u32x{}_unp", elm_per_blk), Span::call_site());

  // Define a match tree
  let mut match_tree: Vec<TokenTree> = quote! {
    match elm_bit_len {}
  }
  .into_iter()
  .collect();

  if let TokenTree::Group(g_tree) = match_tree.pop().unwrap() {
    let mut gs_tree = g_tree.stream();

    // Arm 0
    // No unpacking occurs at a bit-length of 0
    // Generate zero values
    gs_tree.extend_one(quote! {
      0u8 => {
        // `unp` expected to have an exact length of the block.
        // `fill()` will overwrite the entire defined slice
        unp.fill(0);
      },
    });

    // Cycle through [1, 32) bit-lengths
    for elm_bit_len in 1..32u8 {

      // Define a match arm for current bit-length
      let mut match_arm: Vec<TokenTree> = quote! {
        #elm_bit_len => {},
      }
      .into_iter()
      .collect();

      // Unroll the loop calculation with shift sizes at each iteration.
      // Expect that the specified unpacked block is exactly `elm_per_blk` size.
      if let TokenTree::Group(g) = &match_arm[3] {
        let mut gs = g.stream();
        // --- arm: start

        // unp_ptr_off required to be `usize`
        // One unp_ptr_off increment will offset by a single SIMD vector
        // These start at one because the first SIMD vector was loaded
        let mut unp_ptr_off: usize = 1;
        let mut pck_ptr_off: usize = 1;

        // Create a bit-shift mask with the current arm's bit-length
        let msk_lit = proc_macro2::Literal::u32_suffixed(elm_bit_len as u32);
        gs.extend_one(quote! {
          let msk = set1(((1u32 << #msk_lit) - 1u32) as i32);
        });

        // Iterate through shift operations for unpacking
        for cur in blk_itr(elm_bit_len as usize, elm_per_blk) {
          match cur {
            Itr::Fst => {
              // No shift for 1st SIMD lane
              // 1st SIMD vector starts with first u32 added to first SIMD vector
              gs.extend_one(quote! {
                let mut smd_pck = load(pck_ptr);
                let mut prv = set1(fst as i32);
                let mut cur = add(prv, and(smd_pck, msk));
                store(unp_ptr, cur);
                prv = cur;
              });
            },
            Itr::Mdl{ itm } => {
              let shf_lit = proc_macro2::Literal::i32_suffixed(itm.shf_len as i32);
              match itm.shf_dir {
                Dir::Zro => {
                  // No shift
                  gs.extend_one(quote! {
                    cur = add(prv, and(smd_pck, msk));
                    store(unp_ptr.add(#unp_ptr_off), cur);
                    prv = cur;
                  });
                  unp_ptr_off += 1;
                },
                Dir::Bck => {
                  // Partial left shift
                  let shf_lit = proc_macro2::Literal::i32_suffixed(itm.shf_len as i32);
                  gs.extend_one(quote! {
                    cur = add(prv, or(dlt, and(lft(smd_pck, #shf_lit), msk)));
                    store(unp_ptr.add(#unp_ptr_off), cur);
                    prv = cur;
                  });
                  unp_ptr_off += 1;
                },
                Dir::Fwd => {
                  // Full right shift
                  let shf_lit = proc_macro2::Literal::i32_suffixed(itm.shf_len as i32);
                  gs.extend_one(quote! {
                    cur = add(prv, and(rht(smd_pck, #shf_lit), msk));
                    store(unp_ptr.add(#unp_ptr_off), cur);
                    prv = cur;
                  });
                  unp_ptr_off += 1;
                },
                Dir::FwdPrt => {
                  // Partial right shift
                  let shf_lit = proc_macro2::Literal::i32_suffixed(itm.shf_len as i32);
                  gs.extend_one(quote! {
                    let dlt = and(rht(smd_pck, #shf_lit), msk);
                  });
                },
              }

              // Load packed SIMD vector
              if itm.lne_bit_sum == BIT_PER_LNE {
                gs.extend_one(quote! {
                  smd_pck = load(pck_ptr.add(#pck_ptr_off));
                });
                pck_ptr_off += 1;
              }
            },
            Itr::Lst{ itm } => {
              let shf_lit = proc_macro2::Literal::i32_suffixed(itm.shf_len as i32);
              match itm.shf_dir {
                Dir::Zro => {
                  // No shift
                  gs.extend_one(quote! {
                    store(
                      unp_ptr.add(#unp_ptr_off), 
                      add(prv, and(smd_pck, msk)),
                    );
                  });
                },
                Dir::Bck => {
                  // Partial left shift
                  gs.extend_one(quote! {
                    store(
                      unp_ptr.add(#unp_ptr_off), 
                      add(prv, or(dlt, and(lft(smd_pck, #shf_lit), msk))),
                    );
                  });
                },
                Dir::Fwd | Dir::FwdPrt => {
                  // Full right shift or Partial right shift
                  gs.extend_one(quote! {
                    store(
                      unp_ptr.add(#unp_ptr_off), 
                      add(prv, and(rht(smd_pck, #shf_lit), msk)),
                    );
                  });
                },
              }
            },
          }
        }

        // --- arm: end
        // Push unrolled loop into arm definition
        match_arm[3] = Group::new(g.delimiter(), gs.into()).into();
      }

      // Push the match arm onto the match tree
      gs_tree.extend(match_arm);
    }

    // Arm 32
    // No unpacking occurs at a bit-length of 32
    // Copy raw bytes
    gs_tree.extend_one(quote! {
      32u8 => {
        ptr::copy_nonoverlapping(
          pck.as_ptr(), 
          unp.as_mut_ptr() as *mut u8, 
          pck.len());
      },
      _ => panic!("unsupported bit-length {}", elm_bit_len)
    });

    // Push new elements into tree
    match_tree.push(Group::new(g_tree.delimiter(), gs_tree.into()).into());
  }
  let match_arms: proc_macro2::TokenStream = match_tree.into_iter().collect();  

  // Create the unpack method
  return quote! {
    pub unsafe fn #unp_name(elm_bit_len: u8, fst: u32, pck: &[u8], unp: &mut [u32]) {
      let pck_ptr = pck.as_ptr() as *const m256;
      let unp_ptr = unp.as_mut_ptr() as *mut m256;

      #match_arms
    }
  };
}

// u32_blk_bit creates a u32 bit-length method as a TokenStream.
fn u32_blk_bit(elm_per_blk: usize, smd_per_blk: usize) -> proc_macro2::TokenStream {
  // Create the method name
  let fn_name = proc_macro2::Ident::new(&format!("u32x{}_bit_len", elm_per_blk), Span::call_site());

  // Create the pack method
  let mut bit_method: Vec<TokenTree> = quote! {
    #[inline]
    pub unsafe fn #fn_name(blk: &[u32]) -> u8 {
      let mut acm = 0u32;
    }
  }
  .into_iter()
  .collect();

  // Unroll the loop calculation with shift sizes at each iteration.
  // Expect that the specified unpacked block is exactly `elm_per_blk` size.
  if let TokenTree::Group(g) = bit_method.pop().unwrap() {
    let mut gs = g.stream();

    // Use "n" as a multiplier for the starting index with "n * ELM_PER_SMD"
    for n in 0..(smd_per_blk-1) {
      let prv_idx: usize = n * ELM_PER_SMD;
      let cur_idx: usize = (n + 1) * ELM_PER_SMD;
      gs.extend_one(quote! {
        // Load the previous SIMD vector from an array
        let prv_slc = &blk[#prv_idx..#prv_idx + #ELM_PER_SMD];
        let prv = u32x8::from_array(*(prv_slc.as_ptr() as *const [u32; ELM_PER_SMD]));

        // Load the current SIMD vector from an array
        let cur_slc = &blk[#cur_idx..#cur_idx + #ELM_PER_SMD];
        let cur = u32x8::from_array(*(cur_slc.as_ptr() as *const [u32; ELM_PER_SMD]));

        // Delta encode and bitwise accumulate
        acm |= (cur - prv).horizontal_or();

        // There is no performance difference for loading "prv" and "cur" into variables
        // based on Criterion micro benchmarks
      });
    }

    gs.extend_one(quote! {
      // Determine the number of least significant bits used
      return (32u32 - u32::leading_zeros(acm)) as u8;
    });

    // Push unrolled loop into method definition
    let gg = Group::new(g.delimiter(), gs.into());
    bit_method.push(gg.into());
  }

  return bit_method.into_iter().collect();
}

// u32_blk_byt creates a byte-length method as a TokenStream.
fn u32_blk_byt(elm_per_blk: usize) -> proc_macro2::TokenStream {
  // Create the method name
  let fn_name = proc_macro2::Ident::new(&format!("u32x{}_byt_len", elm_per_blk), Span::call_site());

  // Define a match tree
  let mut match_tree: Vec<TokenTree> = quote! {
    match elm_bit_len {
      
    }
  }
  .into_iter()
  .collect();

  // Unroll the loop calculation with shift sizes at each iteration.
  // Expect that the specified unpacked block is greater than or equal to `elm_per_blk` size.
  if let TokenTree::Group(g) = match_tree.pop().unwrap() {
    let mut gs = g.stream();

    // Set the zero bit length to have a byte length of zero
    gs.extend_one(quote! {
      0u8 => 0usize,
    });

    // Cycle through [1, 32) bit-lengths
    for elm_bit_len in 1..32u8 {

      // Initialize the byte length to zero
      let mut byt_len: usize = 0;

      // Iterate through shift operations for packing
      for cur in blk_itr(elm_bit_len as usize, elm_per_blk) {
        match cur {
          Itr::Fst => {},
          Itr::Mdl{ itm } => {
            // Packed SIMD vector is written when lane is full
            if itm.lne_bit_sum == BIT_PER_LNE {
              byt_len += BYT_PER_SMD;
            }
          }
          Itr::Lst{ itm } => {
            // Last iteration always writes a SIMD vector
            // That SIMD vector may be partially packed or fully packed
            byt_len += BYT_PER_SMD;
          },
        }
      }

      // Write the pre-computed byte-length within the match arm
      gs.extend_one(quote! {
        #elm_bit_len => #byt_len,
      });
    }

    // Set the maximum byte length when the element bit-length is 32
    let byt_len: usize = BYT_PER_ELM * elm_per_blk;
    gs.extend_one(quote! {
      32u8 => #byt_len,
      _ => panic!("unsupported bit-length {}", elm_bit_len)
    });

    // Push unrolled loop into method definition
    let gg = Group::new(g.delimiter(), gs.into());
    match_tree.push(gg.into());
  }
  let match_arms: proc_macro2::TokenStream = match_tree.into_iter().collect();

  // Create the byte-length method
  return quote! {
    #[inline]
    pub fn #fn_name(elm_bit_len: u8) -> usize {
      #match_arms
    }
  };
}

struct BlkMacro {
  elm_per_blk_lit: LitInt,
}
impl Parse for BlkMacro {
  fn parse(input: ParseStream) -> Result<Self> {
    Ok(BlkMacro {
      elm_per_blk_lit: input.parse().expect("first parameter `elm_per_blk`"),
    })
  }
}
#[proc_macro]
pub fn u32_blk(input: TokenStream) -> TokenStream {
  // Parse macro syntax paramaters
  let BlkMacro { elm_per_blk_lit } = parse_macro_input!(input as BlkMacro);

  // Parse value of elm_per_blk
  let elm_per_blk = elm_per_blk_lit
    .base10_parse::<usize>()
    .expect("can't parse `elm_per_blk` as usize");

  // Validate min of elm_per_blk
  if elm_per_blk < MIN_ELM_PER_BLK {
    elm_per_blk_lit
      .span()
      .unwrap()
      .error(format!(
        "parameter `elm_per_blk` is too small (min {})",
        MIN_ELM_PER_BLK
      ))
      .emit();
    return TokenStream::new();
  }

  // Validate multiple of elm_per_blk
  if elm_per_blk % ELM_PER_SMD != 0 {
    elm_per_blk_lit
      .span()
      .unwrap()
      .error(format!(
        "parameter `elm_per_blk` is not a multiple of {}",
        ELM_PER_SMD
      ))
      .emit();
    return TokenStream::new();
  }

  // Calculate smd_per_blk
  let smd_per_blk = elm_per_blk / ELM_PER_SMD;

  // Create the pack method
  let pck_method = u32_blk_pck(elm_per_blk, smd_per_blk);

  // Create the unpack method
  let unp_method = u32_blk_unp(elm_per_blk, smd_per_blk);

  // Create the bit-length method
  let bit_method = u32_blk_bit(elm_per_blk, smd_per_blk);

  // Create the byte-length method
  let byt_method = u32_blk_byt(elm_per_blk);

  // Expand all methods
  let expanded = quote! {
    #pck_method

    #unp_method

    #bit_method

    #byt_method
  };

  TokenStream::from(expanded)
}

