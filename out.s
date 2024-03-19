.global _start
.align 2
_start:
MOV X1, #10
STP X1, X2, [SP, #-16]!
MOV X1, #20
STP X1, X2, [SP, #-16]!
MOV X1, #30
STP X1, X2, [SP, #-16]!
MOV X1, #2
STP X1, X2, [SP, #-16]!
MOV X1, #2
STP X1, X2, [SP, #-16]!
MOV X1, #5
STP X1, X2, [SP, #-16]!
LDP X1, X2, [SP, #16]
LDP X2, X3, [SP]
MUL X1, X1, X2
STP X1, X2, [SP, #-16]!
LDP X1, X2, [SP, #48]
LDP X2, X3, [SP]
ADD X1, X1, X2
STP X1, X2, [SP, #-16]!
MOV X1, #10
STP X1, X2, [SP, #-16]!
MOV X1, #2
STP X1, X2, [SP, #-16]!
MOV X1, #3
STP X1, X2, [SP, #-16]!
LDP X1, X2, [SP, #16]
LDP X2, X3, [SP]
MUL X1, X1, X2
STP X1, X2, [SP, #-16]!
LDP X1, X2, [SP, #48]
LDP X2, X3, [SP]
SUBS X1, X1, X2
STP X1, X2, [SP, #-16]!
MOV X1, #2
STP X1, X2, [SP, #-16]!
LDP X1, X2, [SP, #16]
LDP X2, X3, [SP]
SDIV X1, X1, X2
STP X1, X2, [SP, #-16]!
LDP X1, X2, [SP, #0]
STP X1, X2, [SP, #-16]!
LDP X0, X2, [SP]
mov X16, #1
svc #0x80
