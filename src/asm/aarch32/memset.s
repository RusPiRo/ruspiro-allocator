/*********************************************************************************************************************** 
 * Copyright (c) 2019 by the authors
 * 
 * Author: André Borrmann 
 * License: Apache License 2.0
 **********************************************************************************************************************/

// fast memset implementation for propperly aligned and amount of memory
.global __qmset		//(char* trg, u32 value, u32 fastSize)

__qmset: //(uTrg, value, uFastSize);
	push 	{r2-r9, lr}
	mov	r4, r0
	mov	r6, r1
	mov	r7, r1
	mov	r8, r1
	mov	r9, r1
.loops:
	stmia	r4!, {r6-r9}
	subs	r2, #16
	bhi	.loops

	pop	{r2-r9, pc}
