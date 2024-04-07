MEMORY {
   PROGROM (RX) : ORIGIN = 0x00000, LENGTH = 2K
   DATARAM (RW) : ORIGIN = LENGTH(PROGROM), LENGTH = 1K
}

__sp = ORIGIN(DATARAM) + LENGTH(DATARAM); 

SECTIONS {
    .text : {
    . = ALIGN(4);
        *(.text.init);
        *(.text);
        *(.text.*);
    } > PROGROM

    .data : {
	. = ALIGN(4);
        *(.data*)          
        *(.sdata*)
        *(.rodata*) 
        *(.srodata*)
        *(.bss*)
        *(.sbss*)
	
        *(COMMON)
        *(.eh_frame)  
        *(.eh_frame_hdr)
        *(.init_array*)         
        *(.gcc_except_table*)  
    } > DATARAM
}


