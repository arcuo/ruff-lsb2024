
# ifprinciples { alice, bob }


## var = opt

# IF001: Success, opt are considered public variables
public_var = 0 # iflabel {}
bob_var = 42 # iflabel {bob}
alice_bob_var = 69 # iflabel {alice,bob}

## var = var

# IF001: Success - Information flow from {bob} to {alice,bob}. 
alice_bob_var = bob_var

# IF001: Fail - Information flow from {alice,bob} to {bob}
bob_var = alice_bob_var

# IF001: Fail - public var are less restrictive than alice_bob_var
public_var = alice_bob_var

# IF001: Fail - public var are less restrictive than bob_var
public_var = bob_var

# IF001: Success - Public variables can flow to more restrictive variables
alice_bob_var = public_var
bob_var = public_var


# ## var = expr 

# # IF001: Success - opt are considered public variables
alice_bob_var = 69 + 1

# # IF001: Fail - Information flow from a to b
alice_bob_var = bob_var + 1

# # IF001: Success Information flow from b to a
bob_var = alice_bob_var + 1


# ## Augmented assignment - i.e "+="

# # IF001: Success - opt are considered public variables
# alice_bob_var += 1

# # IF001: Fail - Information flow from a to b
# alice_bob_var += bob_var

# # IF001: Success Information flow from b to a
# bob_var += alice_bob_var


## TODO: tuples, lists, dicts, sets

## TODO: var = func() Might be a separate rule
