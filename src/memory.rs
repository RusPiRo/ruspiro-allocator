/***************************************************************************************************
 * Copyright (c) 2019 by the authors
 *
 * Author: André Borrmann
 * License: Apache License 2.0
 **************************************************************************************************/

//! # Lock Free Memory Management
//!

use core::sync::atomic::{AtomicUsize, Ordering};

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
}

/// Need to place the enum values also in an array to be able to iterate over them :/
const BUCKET_SIZES: [MemBucketSize; 15] = [
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

/// The global pointer to the next free memory location on the HEAP not considering re-usage. If no
/// re-usable bucket exists, memory will be allocated at this position. It's implemented as
/// ``usize`` to ensure we can perform immediate atomic math operation (add/sub) on it.
static HEAP_START: AtomicUsize = AtomicUsize::new(0);

/// The list of buckets that may countain re-usable memory blocks
static FREE_BUCKETS: [AtomicUsize; 16] = [
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
];

/// Allocate an arbitrary size of memory on the HEAP
pub(crate) fn alloc(req_size: usize, alignment: usize) -> *mut u8 {
    // if the HEAP START is initial (0) set the address from the linker script
    HEAP_START.compare_and_swap(
        0,
        unsafe { &__heap_start as *const usize as usize },
        Ordering::AcqRel,
    );

    // calculate the required size to be allocated including descriptor size and alignment
    let padding = 1 << alignment;
    let admin_size = core::mem::size_of::<MemoryDescriptor>() + padding;
    // calculate the physical size in memory that is required to be allocated
    let phys_size = admin_size + req_size;

    // the physical size defines the bucket this allocation will fall into, so get the last bucket
    // where this size would fit
    let bucket_idx = BUCKET_SIZES
        .iter()
        .position(|&bucket| phys_size < bucket as usize);

    // if a bucket could be found allocate its size, otherwise allocate the physical size as this is
    // a to large and therefore generic allocation w/o a bucket assignment
    let alloc_size = bucket_idx.map_or(phys_size, |b| BUCKET_SIZES[b] as usize);
    let bucket = bucket_idx.unwrap_or_else(|| BUCKET_SIZES.len());

    // check if we can get the nex position to allocate memory from a re-usable bucket.
    // if this is not the case we retrieve this from the end of the current heap. Both is crucial to
    // get right in the concurrent access scenario
    let descriptor_addr = get_free_bucket(bucket)
        .unwrap_or_else(|| HEAP_START.fetch_add(alloc_size, Ordering::SeqCst));

    assert!(descriptor_addr < 0x3f00_0000);
    // any other concurrent allocation will now see the new HEAP_START, so we can now maintain the
    // descriptor at the given location
    let descriptor = unsafe { &mut *(descriptor_addr as *mut MemoryDescriptor) };

    // now fill the memory descriptor managing this allocation
    descriptor.magic = MM_MAGIC;
    descriptor.bucket = bucket_idx.unwrap_or_else(|| BUCKET_SIZES.len());
    descriptor.size = alloc_size;
    descriptor.align = alignment;
    descriptor.prev = 0;
    descriptor.next = 0;
    descriptor.payload_addr =
        (descriptor as *const MemoryDescriptor as usize + admin_size) & !(padding - 1);
    descriptor._placeholder = 0;

    // the usable address is stored in the payload attribute of the descritor however,
    // while releasing memory the address of this usable location is given, so we need a way to get
    // from this location we need to be able to calculate the MemoryDescriptor location
    // this is done by keeping at least 1 ``usize`` location free in front of the usage memory location
    // and store the descriptor address there
    let descriptor_link_store = descriptor.payload_addr - core::mem::size_of::<usize>();
    unsafe { *(descriptor_link_store as *mut usize) = descriptor_addr };
    assert!(descriptor.payload_addr < 0x3f00_0000);
    let addr = descriptor.payload_addr - core::mem::size_of::<usize>();
    let check = unsafe { *(addr as *const usize) };
    assert_eq!(descriptor_addr, check);
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
    // updating the heap pointer is the critical part here for cocurrent access. So once this happened
    // this location might be used for allocations. So we shall never ever access parts of this location
    // any more if the swap was successfull
    let prev_heap_start =
        HEAP_START.compare_and_swap(heap_check, descriptor_addr, Ordering::SeqCst);
    if prev_heap_start == heap_check {
        // we are done
        return;
    }
    // it's not a memory region at the end of the heap, so put it into the corresponding bucket
    // setting this bucket as the new last free entry is the crucial operation in concurrent access.
    // as soon as this happened any other access sees the new entry
    // as we need to set the previous bucket in the new one while ensuring concurrent access is not
    // re-using this block while doing so we need to do this in steps until we set the new free bucket
    loop {
        // 1. load the previous free bucket
        let prev_free_bucket = FREE_BUCKETS[descriptor.bucket].load(Ordering::Acquire);
        // 2. store this address in the new free bucket
        descriptor.prev = prev_free_bucket;
        // 3. swap the old and the new free bucket if the old free bucket is still the same
        let prev_free_bucket_check = FREE_BUCKETS[descriptor.bucket].compare_and_swap(
            prev_free_bucket,
            descriptor_addr,
            Ordering::SeqCst,
        );
        // 4. if the free bucket was different re-try as it has been occupied in the meanwhile
        if prev_free_bucket == prev_free_bucket_check {
            return;
        }
    }
}

// get the next free re-usable bucket to allocate the memory from
#[inline]
fn get_free_bucket(bucket: usize) -> Option<usize> {
    assert!(bucket < FREE_BUCKETS.len());
    // first check if we have re-usable memory available in the corresponding bucket
    let reusable_bucket = FREE_BUCKETS[bucket].load(Ordering::Acquire);
    // if this is available use it as the free slot, so replace this free bucket with it's previous
    // one. This is crucial in cuncurrent access so do this only if this still is the same free bucket
    if reusable_bucket != 0 {
        let descriptor = unsafe { &*(reusable_bucket as *const MemoryDescriptor) };
        let reusable_bucket_check = FREE_BUCKETS[bucket].compare_and_swap(
            reusable_bucket,
            descriptor.prev,
            Ordering::SeqCst,
        );
        if reusable_bucket_check == reusable_bucket {
            // use the reusable bucket as new memory block
            return Some(reusable_bucket);
        } else {
            // the re-usable bucket has been accupied since the last read, so continue with
            // allocating from the heap
            return None;
        }
    }

    None
}
