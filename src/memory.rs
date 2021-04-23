/***********************************************************************************************************************
 * Copyright (c) 2020 by the authors
 *
 * Author: André Borrmann <pspwizard@gmx.de>
 * License: Apache License 2.0 / MIT
 **********************************************************************************************************************/

//! # Lock Free Memory Management
//!

use core::sync::atomic::{AtomicUsize, Ordering};
//use ruspiro_console::*;

/// The magic identifier for a managed memory block
const MM_MAGIC: u32 = 0xDEAD_BEEF;

/// Memory allocations happens in predefined chunk sizes. This might lead to memory wast in some cases
/// but this could help increasing the speed for re-usage of freed memory regions as we know which
/// bucket to look for when re-using. Memory requirements above 1MB are handled individually w/o any
/// bucket assignment
#[repr(u32)]
#[derive(Copy, Clone, Debug)]
enum MemBucketSize {
  _64B = 0x00_0040,
  _128B = 0x00_0080,
  _256B = 0x00_0100,
  _512B = 0x00_0200,
  _1KB = 0x00_0400,
  _2KB = 0x00_0800,
  _4KB = 0x00_1000,
  _8KB = 0x00_2000,
  _16KB = 0x00_4000,
  _32KB = 0x00_8000,
  _64KB = 0x01_0000,
  _128KB = 0x02_0000,
  _256KB = 0x04_0000,
  _512KB = 0x08_0000,
  _1MB = 0x10_0000,
  _2MB = 0x20_0000,
}

/// Need to place the enum values also in an array to be able to iterate over them :/
const BUCKET_SIZES: [MemBucketSize; 16] = [
  MemBucketSize::_64B,
  MemBucketSize::_128B,
  MemBucketSize::_256B,
  MemBucketSize::_512B,
  MemBucketSize::_1KB,
  MemBucketSize::_2KB,
  MemBucketSize::_4KB,
  MemBucketSize::_8KB,
  MemBucketSize::_16KB,
  MemBucketSize::_32KB,
  MemBucketSize::_64KB,
  MemBucketSize::_128KB,
  MemBucketSize::_256KB,
  MemBucketSize::_512KB,
  MemBucketSize::_1MB,
  MemBucketSize::_2MB,
];

extern "C" {
  /// Linker Symbol which address points to the HEAP START.
  /// Access as &__heap_start -> address!
  static __heap_start: usize;
  /// Linker Symbol which address points to the HEAP END . On a Raspberry Pi this should be treated with
  /// care as the whole HEAP is shared between the ARM CPU and GPU. Only a mailbox call can provide
  /// the real ARM HEAP size
  static __heap_end: usize;
}

/// Descriptive block of a managed memory reagion. This administrative data is stored along side with
/// the actual memory allocated. This means the physical memory requirement is always the requested
/// one + the size of this descriptor
#[repr(C, packed)]
#[derive(Copy, Clone, Default, Debug)]
struct MemoryDescriptor {
  /// The magic of this block
  magic: u32,
  /// The bucket index this memory block is assigned to
  bucket: usize,
  /// The real occupied memory size (descriptor size + payload size)
  size: usize,
  align: usize,
  /// Address of the preceding memory block when this one is ready for re-use
  prev: usize,
  /// Address of the following memory block when this one is ready for re-use
  next: usize,
  /// payload address. In addition the address of the descritor managing this memory need to be
  /// stored relative to the address stored here to ensure we can calculate the descriptor address
  /// back from the payload address in case we were ask to free this location
  payload_addr: usize,
  /// this placeholder ensures that the payload starts earliest after this usize field. If this is
  /// the case this field will contain the address of the descriptor which need to be stored relative
  /// to the payload start address
  _placeholder: usize,
}

struct BucketQueue {
  head: AtomicUsize,
  tail: AtomicUsize,
}

/// The global pointer to the next free memory location on the HEAP not considering re-usage. If no
/// re-usable bucket exists, memory will be allocated at this position. It's implemented as
/// ``usize`` to ensure we can perform immediate atomic math operation (add/sub) on it.
static HEAP_START: AtomicUsize = AtomicUsize::new(0);

/// The list of buckets that may contain re-usable memory blocks. The new free memory blocks are added always to the
/// tail of each list, while the retrival always happens from the head. Like FIFO buffer
static FREE_BUCKETS: [BucketQueue; BUCKET_SIZES.len() + 1] = [
  BucketQueue {
    head: AtomicUsize::new(0),
    tail: AtomicUsize::new(0),
  },
  BucketQueue {
    head: AtomicUsize::new(0),
    tail: AtomicUsize::new(0),
  },
  BucketQueue {
    head: AtomicUsize::new(0),
    tail: AtomicUsize::new(0),
  },
  BucketQueue {
    head: AtomicUsize::new(0),
    tail: AtomicUsize::new(0),
  },
  BucketQueue {
    head: AtomicUsize::new(0),
    tail: AtomicUsize::new(0),
  },
  BucketQueue {
    head: AtomicUsize::new(0),
    tail: AtomicUsize::new(0),
  },
  BucketQueue {
    head: AtomicUsize::new(0),
    tail: AtomicUsize::new(0),
  },
  BucketQueue {
    head: AtomicUsize::new(0),
    tail: AtomicUsize::new(0),
  },
  BucketQueue {
    head: AtomicUsize::new(0),
    tail: AtomicUsize::new(0),
  },
  BucketQueue {
    head: AtomicUsize::new(0),
    tail: AtomicUsize::new(0),
  },
  BucketQueue {
    head: AtomicUsize::new(0),
    tail: AtomicUsize::new(0),
  },
  BucketQueue {
    head: AtomicUsize::new(0),
    tail: AtomicUsize::new(0),
  },
  BucketQueue {
    head: AtomicUsize::new(0),
    tail: AtomicUsize::new(0),
  },
  BucketQueue {
    head: AtomicUsize::new(0),
    tail: AtomicUsize::new(0),
  },
  BucketQueue {
    head: AtomicUsize::new(0),
    tail: AtomicUsize::new(0),
  },
  BucketQueue {
    head: AtomicUsize::new(0),
    tail: AtomicUsize::new(0),
  },
  BucketQueue {
    head: AtomicUsize::new(0),
    tail: AtomicUsize::new(0),
  },
];

/// Allocate an arbitrary size of memory on the HEAP
/// The alignment is given in Bytes and need to be a power of 2
pub(crate) fn alloc(req_size: usize, alignment: usize) -> *mut u8 {
  // if the HEAP START is initial (0) set the address from the linker script
  let _ = HEAP_START.compare_exchange(
    0,
    unsafe { &__heap_start as *const usize as usize },
    Ordering::AcqRel,
    Ordering::Release,
  );

  // calculate the required size to be allocated including descriptor size and alignment
  let padding = alignment; //1 << alignment;
  let admin_size = core::mem::size_of::<MemoryDescriptor>() + padding;
  // calculate the physical size in memory that is required to be allocated
  let phys_size = admin_size + req_size;

  // the physical size defines the bucket this allocation will fall into, so get the smallest bucket
  // where this size would fit
  let bucket_idx = BUCKET_SIZES
    .iter()
    .position(|&bucket| phys_size < bucket as usize);

  // if a bucket could be found allocate its size, otherwise allocate the requested size w/o a bucket assignment
  let alloc_size = bucket_idx.map_or(phys_size, |b| BUCKET_SIZES[b] as usize);
  let bucket = bucket_idx.unwrap_or_else(|| BUCKET_SIZES.len());

  // check if we can get the next position to allocate memory from a re-usable bucket.
  // if this is not the case we retrieve this from the end of the current heap. Both is crucial to
  // get right in the concurrent/multicore access scenario
  let descriptor_addr = pop_from_free_bucket(bucket, alloc_size)
    .unwrap_or_else(|| HEAP_START.fetch_add(alloc_size, Ordering::SeqCst));
  //let descriptor_addr = HEAP_START.fetch_add(alloc_size, Ordering::SeqCst);

  assert!(descriptor_addr < 0x3f00_0000);
  // any other concurrent allocation will now see the new HEAP_START, so we can now maintain the
  // descriptor at the given location
  let descriptor = unsafe { &mut *(descriptor_addr as *mut MemoryDescriptor) };

  // now fill the memory descriptor managing this allocation
  descriptor.magic = MM_MAGIC;
  descriptor.bucket = bucket;
  descriptor.size = alloc_size;
  descriptor.align = alignment;
  descriptor.prev = 0;
  descriptor.next = 0;
  descriptor._placeholder = 0;
  descriptor.payload_addr = (descriptor_addr + admin_size) & !(padding - 1);
  assert!(descriptor.payload_addr > descriptor_addr + core::mem::size_of::<MemoryDescriptor>());

  // the usable address is stored in the payload attribute of the descriptor, however,
  // while releasing memory with this address given, we need a way to calculate the MemoryDescriptor location from
  // there. This is done by keeping at least 1 ``usize`` location free in front of the usage
  // memory location and store the descriptor address there
  let descriptor_link_store = descriptor.payload_addr - core::mem::size_of::<usize>();
  unsafe { *(descriptor_link_store as *mut usize) = descriptor_addr };
  // now hand out the actual payload address pointing to the allocated memory with at least the requested size
  descriptor.payload_addr as *mut u8
}

/// allocate memory in chunks of pages, where the page size depends on the architecture and is therefore given from the
/// caller. It always allocates memory that is alligned to the page boundaries and occupies (num * page_size) memory on
/// the heap
#[allow(dead_code)]
pub(crate) fn alloc_page(num: usize, page_size: usize) -> *mut u8 {
  // for the time beeing we will always allocate fresh memory from the heap for this kind of allocation
  // and do never check available free buckets
  // if the HEAP START is initial (0) set the address from the linker script
  let _ = HEAP_START.compare_exchange(
    0,
    unsafe { &__heap_start as *const usize as usize },
    Ordering::AcqRel,
    Ordering::Release,
  );

  // from the current HEAP_START calculate the next start address of a page
  let mut heap_start = HEAP_START.load(Ordering::Acquire);
  let heap_align = (heap_start + page_size - 1) & !(page_size - 1);
  // if the aligned address does not allow enough space for the memory descriptor we need
  // "waste" some memory and go to the next page start address
  if (heap_align - heap_start) < core::mem::size_of::<MemoryDescriptor>() {
    heap_start = heap_align + page_size;
  } else {
    heap_start = heap_align;
  }

  // as we now know where the requested memory will start and end we can update the HEAP_START accordingly to let
  // others know where to request memory from
  HEAP_START.store(heap_start + num * page_size, Ordering::Release); // from her other cores will be able to access
                                                                     // this as well

  let alloc_size = num * page_size + core::mem::size_of::<MemoryDescriptor>();
  let descriptor_addr = heap_start - core::mem::size_of::<MemoryDescriptor>();
  // fill the descriptor structure
  let descriptor = unsafe { &mut *(descriptor_addr as *mut MemoryDescriptor) };

  // now fill the memory descriptor managing this allocation
  descriptor.magic = MM_MAGIC;
  descriptor.bucket = BUCKET_SIZES.len();
  descriptor.size = alloc_size;
  descriptor.align = page_size;
  descriptor.prev = 0;
  descriptor.next = 0;
  descriptor._placeholder = 0;
  descriptor.payload_addr = heap_start;
  assert!(descriptor.payload_addr < 0x3f00_0000);

  // the usable address is stored in the payload attribute of the descriptor, however,
  // while releasing memory with this address given, we need a way to calculate the MemoryDescriptor location from
  // there. This is done by keeping at least 1 ``usize`` location free in front of the usage
  // memory location and store the descriptor address there
  let descriptor_link_store = descriptor.payload_addr - core::mem::size_of::<usize>();
  unsafe { *(descriptor_link_store as *mut usize) = descriptor_addr };
  //info!("{:#x?} -> {:#x?}, linkstore: {:#x?}", descriptor_addr, descriptor, descriptor_link_store);
  // now hand out the actual payload address pointing to the allocated memory with at least the requested size
  descriptor.payload_addr as *mut u8
}

/// Free the memory occupied by the given payload pointer
pub(crate) fn free(address: *mut u8) {
  // first get the address of the descriptor for this payload pointer
  let descriptor_link_store = (address as usize) - core::mem::size_of::<usize>();
  let descriptor_addr = unsafe { *(descriptor_link_store as *const usize) };
  let mut descriptor = unsafe { &mut *(descriptor_addr as *mut MemoryDescriptor) };
  assert!(descriptor.magic == MM_MAGIC);
  // clean the magic of this memory block
  descriptor.magic = 0;
  // we now know the data of this memory descriptor, add this one to the corresponding free bucket
  // or just adjust the heap pointer if this is the last memory entry that is about to be freed
  let heap_check = descriptor_addr + descriptor.size;
  // updating the heap pointer is the critical part here for concurrent access. So once this happened
  // this location might be used for allocations. So we shall never ever access parts of this location
  // any more if the swap was successfull
  if HEAP_START
    .compare_exchange(
      heap_check,
      descriptor_addr,
      Ordering::AcqRel,
      Ordering::AcqRel,
    )
    .is_ok()
  {
    // we are done
    return;
  }
  // it's not a memory region at the end of the heap, so put it into the corresponding bucket
  push_to_free_bucket(descriptor);
}

#[inline]
fn push_to_free_bucket(descriptor: &mut MemoryDescriptor) {
  // setting this bucket as the new last free entry is a crucial operation in concurrent access.
  // as soon as this happened any other access sees the new entry
  // as we need to set the previous bucket in the new one while ensuring concurrent access is not
  // re-using this block while doing so we need to do this in steps until we set the new free bucket
  let descriptor_addr = descriptor as *mut MemoryDescriptor as usize;
  loop {
    // 1. load the previous free bucket
    let prev_free_bucket = FREE_BUCKETS[descriptor.bucket].tail.load(Ordering::Acquire);
    // 2. store this address in the new free bucket
    descriptor.prev = prev_free_bucket;
    descriptor.next = 0;
    // 3. swap the old and the new free bucket if the old free bucket is still the same
    if FREE_BUCKETS[descriptor.bucket]
      .tail
      .compare_exchange(
        prev_free_bucket,
        descriptor_addr,
        Ordering::AcqRel,
        Ordering::Release,
      )
      .is_ok()
    {
      // 5. if we have successfully pushed this to the tail, update the next pointer in the previous
      // descriptor to make the chain complete
      if prev_free_bucket != 0 {
        let prev_descriptor = unsafe { &mut *(prev_free_bucket as *mut MemoryDescriptor) };
        prev_descriptor.next = descriptor_addr;
      } else {
        // 6. if the previous free bucket was not set the head is also not set, so update the head to the new
        // free bucket as well
        FREE_BUCKETS[descriptor.bucket]
          .head
          .store(descriptor_addr, Ordering::SeqCst);
      }
      return;
    }
  }
}

// get the next free re-usable bucket to allocate the memory from
#[inline]
fn pop_from_free_bucket(bucket: usize, _alloc_size: usize) -> Option<usize> {
  assert!(bucket < FREE_BUCKETS.len());
  // TODO: dynamically sized buckets need special treatment to see if the requested size will fit into one. This
  // is not yet properly tested, so for the time beeing any dynamically sized freed bucket will never be re-used
  // but memory will always be requested from the HEAP end.
  if bucket == BUCKET_SIZES.len() {
    /*
    // if a free bucket shall be searched from the dynamically sized ones we need to check each free bucket
    // whether the requested memory will fit into it, only then we can re-use this bucket.
    // while doing so atomically "consume" each bucket from the end till the start which will ensure that the same
    // bucket will not be verified twice in a concurrent/multi-core szenario.

    let mut reusable_bucket = FREE_BUCKETS[bucket].load(Ordering::Acquire);
    while reusable_bucket != 0 {
        // we found a re-usable block, "consume" it
        let descriptor = unsafe { &*(reusable_bucket as *const MemoryDescriptor) };
        let reusable_bucket_check = FREE_BUCKETS[bucket].compare_and_swap(
            reusable_bucket,
            descriptor.prev,
            Ordering::Release,
        );
        if reusable_bucket_check == reusable_bucket {
            // we were able to consume the bucket, all others will see the correct free bucket list not interfering
            // with this one - let's check for its size
            if descriptor.size >= alloc_size {
                // this memory block can hold the requested size, check how much memory will be left over to see if
                // we could split the region up into another re-usable memory block
                let remaining_size = descriptor.size -  alloc_size;
                // check only if at least 64Bytes are remaining
                if remaining_size > MemBucketSize::_64B {
                    // create a new memory descriptor located after the memory we re-use
                    let mut descriptor = unsafe {
                        &mut *((reusable_bucket + alloc_size) as *mut MemoryDescriptor)
                    };
                    let bucket_idx = BUCKET_SIZES
                        .iter()
                        .position(|&bucket| remaining_size < bucket as usize);
                    let bucket = bucket_idx.unwrap_or_else(|| BUCKET_SIZES.len());

                    descriptor.bucket = bucket;
                    descriptor.size = remaining_size;
                    push_to_free_bucket(descriptor);
                }
                // now we can return the re-usable bucket
                return Some(reusable_bucket);
            } else {
                // this block does not offer enough space to be re-used by this request
                // so - check the next one

            }

        }
        // as we could not consume this re-usable block (someone else was faster :) ) - try the next one that is
        // actually stored in the list
        reusable_bucket = reusable_bucket_check;
    }
    */
    // no reusable memory block found --> trigger allocation from fresh heap ...
    return None;
  } else {
    // first check if we have re-usable memory available in the corresponding bucket
    let reusable_bucket = FREE_BUCKETS[bucket].head.load(Ordering::Acquire);
    // if this is available use it as the free slot, so replace this free bucket with it's next
    // one. This is crucial in cuncurrent access so do this only if this still is the same free bucket
    if reusable_bucket != 0 {
      let descriptor = unsafe { &*(reusable_bucket as *const MemoryDescriptor) };
      if FREE_BUCKETS[bucket]
        .head
        .compare_exchange(
          reusable_bucket,
          descriptor.next,
          Ordering::Release,
          Ordering::Release,
        )
        .is_ok()
      {
        if descriptor.next != 0 {
          // if we had a next block update it's previous one
          let next_descriptor = unsafe { &mut *(descriptor.next as *mut MemoryDescriptor) };
          next_descriptor.prev = 0;
        } else {
          // clear the tail as this was the last entry in the list
          FREE_BUCKETS[bucket].tail.store(0, Ordering::SeqCst);
        }
        // use the reusable bucket as new memory block
        return Some(reusable_bucket);
      } else {
        // the re-usable bucket has been occupied since the last read, so continue with
        // allocating from the heap
        return None;
      }
    }
  }

  None
}
