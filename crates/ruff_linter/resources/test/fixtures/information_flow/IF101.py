
# ifprinciples { alice, bob }


## var = opt

# IF101: Success, opt are considered public variables
public_var = 0 # iflabel {}
bob_var = 42 # iflabel {bob}
alice_bob_var = 69 # iflabel {alice,bob}

## var = var

# IF101: Success - Information flow from {bob} to {alice,bob}. 
alice_bob_var = bob_var

# IF101: Fail - Information flow from {alice,bob} to {bob}
bob_var = alice_bob_var

# IF101: Fail - public var are less restrictive than alice_bob_var
public_var = alice_bob_var

# IF101: Fail - public var are less restrictive than bob_var
public_var = bob_var

# IF101: Success - Public variables can flow to more restrictive variables
alice_bob_var = public_var
bob_var = public_var


# ## var = expr 

# # IF101: Success - opt are considered public variables
alice_bob_var = 69 + 1

# # IF101: Fail Information flow from b to a
bob_var = alice_bob_var + 1

# # IF101: Success - Information flow from a to b
alice_bob_var = bob_var + 1


## Augmented assignment - i.e "+="

# IF101: Success - opt are considered public variables
alice_bob_var += 1

# IF101: Success - Information flow from a to b
alice_bob_var += bob_var

# IF101: Fail Information flow from b to a
bob_var += alice_bob_var

# IF101: Fail Information flow from b to a
public_var += bob_var

## Annotated assignment - i.e ": int ="

# IF101: Success - opt are considered public variables
alice_bob_var: int = 69

# IF101: Fail Information flow from b to a
bob_var: int = alice_bob_var

# IF101: Success - Information flow from a to b
alice_bob_var: int = bob_var

## Multiple assignment

# IF101: Success - Information flow from {} to {bob}, {alice,bob}
bob_var = alice_bob_var = 1

# IF101: Fail - Information flow from {} to {bob}, {alice,bob}
public_var = bob_var = alice_bob_var

## Tuple assignment

# IF101: Success - Information flow from {} to {bob}
public_var, bob_var = 1, 2
# IF101: Success - Information flow from {} to {alice,bob}
alice_bob_var, bob_var = 1, 2

# IF101: Fail - Information flow from {alice,bob} to {bob}
alice_bob_var, bob_var = 1, alice_bob_var
alice_bob_var, bob_var = alice_bob_var


# List assignment

# IF101: Success - Information flow from literal to {bob}
bob_var = [1, 2, 3]

# IF101: Fail - Information flow from {alice,bob} to {bob}
bob_var = [1, 2, alice_bob_var]

# IF101: Fail - Information flow from {bob} to {public}
public_var = [1, 2, bob_var]

# Tuple assignment

# IF101: Success - Information flow from literal to {bob}
bob_var = (1, 2, 3)

# IF101: Fail - Information flow from {alice,bob} to {bob}
bob_var = (1, 2, alice_bob_var)

# IF101: Fail - Information flow from {bob} to {public}
public_var = (1, 2, bob_var)

# Set assignment

# IF101: Success - Information flow from literal to {bob}
bob_var = {1, 2, 3}

# IF101: Fail - Information flow from {alice,bob} to {bob}
bob_var = {1, 2, alice_bob_var}

# IF101: Fail - Information flow from {bob} to {public}
public_var = {1, 2, bob_var}

# Dict assignment

# IF101: Success - Information flow from literal to {bob}
bob_var = {1: 2, 3: 4}

# IF101: Fail - Information flow from {alice,bob} to {bob}
bob_var = {1: 2, 3: alice_bob_var}

# IF101: Fail - Information flow from {bob} to {public}
public_var = {1: 2, 3: bob_var}

## TODO: var = func() Might be a separate rule
