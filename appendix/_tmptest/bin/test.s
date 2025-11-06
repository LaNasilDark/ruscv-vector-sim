    .section .text
    .global _start

_start:

	# Same destination & WAR
	vsetivli x0, 8, e32, m1, ta, ma # 0x0

	vle32.v v2, (a2)
	vle32.v v3, (a3)
	vle32.v v4, (a4)
	vle32.v v5, (a5)

	vadd.vv v1, v2, v3             
	vmul.vx v6, v1, a0              
	vadd.vv v1, v4, v5      
		   
	vse32.v v6, (a6)                 
	vse32.v v1, (a7)       


	# WAR
	vsetivli x0, 8, e32, m1, ta, ma # 0x28

	vle32.v v2, (a1)
	vle32.v v1, (a0)
	vadd.vv v3, v1, v2       
	vle32.v v1, (a2)    	#overwrites v1 which was just read
	vmul.vv v4, v1, v3  
	vse32.v v4, (a3)         



	# bunch of dependencies & RAW
	vsetivli x0, 8, e32, m1, ta, ma # 0x44

	vle32.v v0, (a0)     
	vle32.v v1, (a1)    

	vmul.vv v2, v0, v1      
	vredsum.vs v3, v2, v3   
	vadd.vx v4, v3, a2  

	vse32.v v4, (a3)   



	# WAW
	vsetivli x0, 8, e32, m1, ta, ma # 0x60

	vle32.v v2, (a0)
	vle32.v v3, (a1)
	vle32.v v4, (a2)

	vmv.v.v v5, v4
	vmul.vv v4, v4, v3	# overwrite
	vsub.vv v4, v4, v2	# overwrite
	vse32.v v4, (a3)


	# Loading same address
	vsetivli x0, 8, e32, m1, ta, ma # 0x80

	vle32.v v1, (a0)
	vle32.v v2, (a1)
	vmul.vv v3, v1, v2
	vse32.v v3, (a2)       
	vle32.v v4, (a2)        # load same addr 
	vadd.vv v5, v3, v4    
	vse32.v v5, (a3) # 0x9c