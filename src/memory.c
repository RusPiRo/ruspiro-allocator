/*********************************************************************************************************************** 
 * Copyright (c) 2019 by the authors
 * 
 * Author: André Borrmann 
 * License: Apache License 2.0
 **********************************************************************************************************************/

// the magic identifying a managed memory block
#define MM_MAGIC	0xDEADBEEF
#define MM_BLOCKS   14

#define	MEM_SIZE_1K		0x000400
#define	MEM_SIZE_2K		0x000800
#define	MEM_SIZE_4K		0x001000
#define	MEM_SIZE_8K		0x002000
#define	MEM_SIZE_16K	0x004000
#define	MEM_SIZE_32K	0x008000
#define	MEM_SIZE_64K	0x010000
#define	MEM_SIZE_128K	0x020000
#define	MEM_SIZE_256K	0x040000
#define	MEM_SIZE_512K	0x080000
#define MEM_SIZE_1M		0x100000

#define MEM_PAGE_SIZE 			MEM_SIZE_4K
#define MEM_PAGE_ALIGN_SHIFT	14
#define MEM_PAGE_MASK 			(~(MEM_PAGE_SIZE-1))
#define MEM_PAGE_ALIGN(x) 		((x + MEM_PAGE_SIZE - 1) & MEM_PAGE_MASK)

#define MEM_SECTION_SIZE		MEM_SIZE_1M
#define MEM_SECTION_ALIGN_SHIFT	20
#define MEM_SECTION_MASK		(~(MEM_SECTION_SIZE-1))
#define MEM_SECTION_ALIGN(x)	((x + MEM_SECTION_SIZE - 1) & MEM_SECTION_MASK)

extern char __heap_start; // the address of this external indicates the start address of the HEAP
extern char __heap_end;	 // the address of this external indicates the end of the HEAP, never allocate memory beyond this point!
extern void __qmset(char* trg, unsigned int value, unsigned int fastSize);
extern void __qcopy(char* trg, const char* src, unsigned int fastSize);

/*
 * each allocated memory block contains this header
 * the address returned by the memory allocation function points to a fixed position
 * after this header. Thus the address of the HEADER can easily calculated when free is called
 */
typedef struct {
	unsigned int 	magic;	  	// indicates that this address really is managed memory
	unsigned int	size;	  	// the size of this allocated memory block
	unsigned int    psize;      // real size of the buffer in memory incl. admin data
	unsigned int   prev;		// address of the preceding block
	unsigned int   next; 		// address of the next block, 0 if at end of heap
	char		data[0];  		// here the data begins, &data is the pointer the malloc functions returns
								// this is also the address m_free will receive for memory releasing
}MEMORY_HEADER_T;

static unsigned int gHeapStart = 0;				// the global pointer to the last position of complete unused memory in the heap
static unsigned int gHeapMax = 0;				// maximum available Heap size
static unsigned int gHeapUsed = 0;				// the total number of bytes occupied on the heap (incl. admin data of each block)
static unsigned int gBlockSizes[] = {0x40, 0x100, 0x400, 0x1000, 0x4000, 0x10000, 0x40000, 0x100000, 0x400000, 0x800000, 0x1000000, 0x4000000, 0x10000000};
static MEMORY_HEADER_T* gFreeList[MM_BLOCKS] = {0,0,0,0,0,0,0,0,0,0,0,0,0,0};		// pointer to the last block of the 2way linked list of free memory chunks ready for re-use
                                                // this is an array of free lists for individual fixed chunk sizes
                                                // we will only allocate memory of specific chunk sizes
                                                // 0x40, 0x100, 0x400, 0x1000, 0x4000, 0x10000, 0x40000, 0x100000, 0x400000, 0x800000, 0x1000000, 0x4000000, 0x10000000
												// 64b , 256b , 1kb  , 4kb   , 16kb  , 64kb   , 256kb  , 1Mb     , 4Mb     , 8Mb     , 16Mb     , 64Mb     , 256Mb

unsigned int m_get_heap_start() {
	return (unsigned int)&__heap_start;
}

unsigned int m_get_heap_end() {
	return (unsigned int)&__heap_end;
}

unsigned int m_get_heap_size() {
	return (unsigned int)&__heap_start - (unsigned int)&__heap_end;
}

/*
 * allocate memory
 */
void* m_alloc(unsigned int size) {
	if (gHeapStart == 0) {
		gHeapStart = (unsigned int)&__heap_start; // initialize the heap pointer at first allocation
		gHeapMax = (unsigned int)&__heap_end - (unsigned int)&__heap_start; // calculate the max. available space
		// TODO: provide a function to set the max heap address from extern as this in on the Raspberry Pi
		// 		 configured in the config.txt file when setting up the memory the GPU should be able to use
		//       but this requires a mailbox call and should not be linked in as dependency to this crate
		unsigned int uSize = gHeapMax;
		// the ARM base address is usually 0
		// so the really available space is the ARM size - heap_start as the program already occupy ARM memory for
		// the executable and the stack
		gHeapMax = uSize - gHeapStart;		
	}

	// calculate the size we need physically on the heap to store the requested data
	// and ensure address alignment to at least 32bit
	unsigned int uAllocSize = (size+sizeof(MEMORY_HEADER_T)+0x1F) & (~0x1F);

	// now calculate the block size
	unsigned int uBlock = 0; // always a minimum memory block will be allocated
	while (uAllocSize > gBlockSizes[uBlock] && uBlock < MM_BLOCKS - 1) uBlock++;
	if (uAllocSize > gBlockSizes[uBlock]) {
		// last block reached - keep alloc size as "free" size
	} else {
		// always allocate full blocks
		uAllocSize = gBlockSizes[uBlock];
	}

	// check that there is enough space left
	if (gHeapUsed + uAllocSize > gHeapMax) {
		return 0;
	}

	MEMORY_HEADER_T* header = 0;
	// check for a free block that is able to cover this memory request
	// the free pointer is always pointing to the last item in the list
	MEMORY_HEADER_T* pFree = gFreeList[uBlock];
	if (pFree) {
		// we found a block that could cover the requested memory
		// release it from the free list and reuse it
		gFreeList[uBlock] = (MEMORY_HEADER_T*)pFree->prev;
		if (gFreeList[uBlock])
			gFreeList[uBlock]->next = 0;
		header = pFree;
	} else {
		// no re-usable memory block available for the requested size
		// allocate a brand new one
		header = (MEMORY_HEADER_T*)gHeapStart;
		// move the heap-start pointer behind the newly used memory
		gHeapStart+=uAllocSize;
	}

	// now we have a new or re-used memory block
	// set the admin data
	header->magic = MM_MAGIC;							// set the magic
	header->size = size;								// available block size to the caller
	header->psize = uAllocSize;							// real memory size
	header->prev = 0;									// allocated memory is not ordered in any list
	header->next = 0;

	// increase used memory
	gHeapUsed+=header->psize;

	return header == 0 ? header : (void *)&header->data;
}

/*
 * allocate aligned memory
 */
void* m_alloca(unsigned int size, short sAlignment) {
	// the admin data will cover at least the size of a pointer
	// and some padding to ensure enough space allocated when aligning the start address
	unsigned int uPadding = (1 << sAlignment)-1;
	unsigned int uAdmin = sizeof(void*) + uPadding;
	// allocate the memory
	void* pRealBlock = m_alloc(size+uAdmin);
	if (pRealBlock == 0) return 0; // not enough space
	// the aligned address is the start address + buffer for real address + padding bytes and than masked with XOR(padding)
	// this returns an aligned address
	// use pointer of pointer for easy access of the storage of where to put the real address
	void** pAlignedBlock = (void **)(((unsigned int)pRealBlock+uAdmin) & ~uPadding);
	// now store the real address before the aligned address
	pAlignedBlock[-1] = pRealBlock;
	return pAlignedBlock;
}

/*
 * release memory allocation
 */
void m_free(void* ptr) {
	if (ptr == 0) return; // nothing to do for 0 pointer
	// get the header data from the pointer to be freed
	MEMORY_HEADER_T* header = (MEMORY_HEADER_T*)((unsigned int)ptr - sizeof(MEMORY_HEADER_T));
	if (header->magic != MM_MAGIC) {
		
	} else {
		// in case this block has been the last in the HEAP
		// we do not put this block into the free list, we just reduce the heap end marker
		if ((unsigned int)header + header->psize == gHeapStart) {
			// clear the magic
			header->magic = 0;
			gHeapStart = (unsigned int)header;
		} else {
			// mark this block as free by adding it to the free list based on it's block size
			// now calculate the block size
			unsigned short uBlock = 0; // always a minimum memory block will be allocated
			while (header->psize > gBlockSizes[uBlock] && uBlock < MM_BLOCKS - 1) uBlock++;
			if (header->psize > gBlockSizes[uBlock]) {
				uBlock = MM_BLOCKS - 1;
			}

			MEMORY_HEADER_T* pLastFree = gFreeList[uBlock];
			if (pLastFree == 0) {
				// the first free block of this size
				gFreeList[uBlock] = header;
				gFreeList[uBlock]->prev = 0;
				gFreeList[uBlock]->next = 0;
			} else {
				pLastFree->next = (unsigned int)header;
				gFreeList[uBlock] = header;
				gFreeList[uBlock]->prev = (unsigned int)pLastFree;
				gFreeList[uBlock]->next = 0;
			}
		}
		gHeapUsed-=header->psize;
	}
}

void m_freea(void* ptr) {
	m_free(((void**)ptr)[-1]);
}

void m_memset(void* trg, const char value, unsigned int size) {
	// check if we can use fast copy
	unsigned int uFastSize = (size >> 4) << 4;
	void* uTrg = trg;
	if (uFastSize) {
		__qmset(trg, value, uFastSize);
	}
	unsigned int uSlowSize = size - uFastSize;
	uTrg+= uFastSize;
	for (int i=0;i<uSlowSize;i++){
		(*(unsigned int volatile *)uTrg) = value;
		uTrg++;
	}
}

unsigned int bcmp(const void* src, const void* trg, unsigned int size) {
	const unsigned char* cSrc = (const unsigned char*)src;
	const unsigned char* cTrg = (const unsigned char*)trg;
	for (unsigned int p = 0; p < size; p++) {
		if (cSrc[p] != cTrg[p]) return p;
	}

	return 0;
}
