#!/bin/bash
ouput_file=out

assembler=as -arch arm64 $ouput_file.s -o $ouput_file.o 
linker=ld -o $ouput_file $ouput_file.o -lSystem -syslibroot `xcrun -sdk macosx --show-sdk-path` -e _start -arch arm64 

cargo run janu.bk \
    && $assembler \
    && $linker \
    && ./out;

echo $?
