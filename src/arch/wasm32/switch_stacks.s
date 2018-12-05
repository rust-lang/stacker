	.text
	.section	.text.__stacker_switch_stacks,"",@

	.functype	typeindex0 (i32) -> ()

	.globl	__stacker_switch_stacks
	.type	__stacker_switch_stacks,@function
__stacker_switch_stacks:                # @__stacker_switch_stacks
	
	.functype	__stacker_switch_stacks (i32, i32, i32) -> ()
	.local  	i32 # temporary storage for the original stack pointer
# %bb.0:                                # %entry
	# save current stack pointer
	get_global	__stack_pointer@GLOBAL
	set_local	3

	# swap stack pointer with the given one
	get_local	0
	set_global	__stack_pointer@GLOBAL

	# call the given function
	get_local	2
	get_local	1
	call_indirect	typeindex0@TYPEINDEX

	# restore sp
	get_local 3
	set_global	__stack_pointer@GLOBAL

	return
	end_function
.Lfunc_end1:
	.size	__stacker_switch_stacks, .Lfunc_end1-__stacker_switch_stacks
										# -- End function

	.globaltype	__stack_pointer, i32
