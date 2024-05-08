
## BAD

b = 0
b2: int = 0
b3 = b4 = 0
b5, b6 = 0, 0
b7, b8 = 1

## GOOD

a1 = 0 # iflabel {}
a2 = a3 = 0 # iflabel {}
a2, a3 = 0, 0 # iflabel {}
a4, a5 = 1 # iflabel {}

## Don't care about augmented assignment or reassignment
a5 += 1
a5 = 2

