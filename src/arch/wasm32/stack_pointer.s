	.text
	.section	.text.__stacker_stack_pointer,"",@
	.globl	__stacker_stack_pointer
	.type	__stacker_stack_pointer,@function
__stacker_stack_pointer:                # @__stacker_stack_pointer
	.functype	__stacker_stack_pointer () -> (i32)
	.local  	i32 # It is workaround for LLVM and serves no other purpose.
# %bb.0:                                # %entry
	get_global	__stack_pointer@GLOBAL
	return
	end_function
.Lfunc_end0:
	.size	__stacker_stack_pointer, .Lfunc_end0-__stacker_stack_pointer
										# -- End function
	.globaltype	__stack_pointer, i32