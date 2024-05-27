
# ifprinciples { alice, bob }

public_var = 0 # iflabel {}
bob_var = 42 # iflabel {bob}
alice_bob_var = 69 # iflabel {alice,bob}

## Tuple assignment

# IF101: Success - Information flow from {} to {bob}
public_var, bob_var = 1, 2
# IF101: Success - Information flow from {} to {alice,bob}
alice_bob_var, bob_var = 1, 2

# IF101: Fail - Information flow from {alice,bob} to {bob}
alice_bob_var, bob_var = 1, alice_bob_var
alice_bob_var, bob_var = alice_bob_var
