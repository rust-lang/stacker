	.text
	.section	.text.__stacker_black_box,"",@
	.globl	__stacker_black_box
	.type	__stacker_black_box,@function
__stacker_black_box:                    # @__stacker_black_box
	.functype	__stacker_black_box (i32) -> ()
	.local  	i32 # It is workaround for LLVM and serves no other purpose.
# %bb.0:                                # %entry
	end_function
.Lfunc_end0:
	.size	__stacker_black_box, .Lfunc_end0-__stacker_black_box
										# -- End function

