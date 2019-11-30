/*********************************************************************************************************************** 
 * Copyright (c) 2019 by the authors
 * 
 * Author: André Borrmann 
 * License: Apache License 2.0
 **********************************************************************************************************************/

// fast memset implementation for propperly aligned and amount of memory
.global __qmset		//(char* trg, u32 value, u32 fastSize)

__qmset: //(uTrg, value, uFastSize);
	mov		x5, x0
	mov		w3, w1
	lsl     x3, x3, #32
	mov     w3, w1
	mov     w4, w1
	lsl     x4, x4, #32
	mov     w4, w1

.loops:
	stp		x3, x4, [x5], #8
	subs	x2, x2, #8
	bhi	.loops

	ret