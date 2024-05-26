
# ifprinciples { alice, bob }

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