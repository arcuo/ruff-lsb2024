# ifprinciples { alice, bob }

public_var = 0 # iflabel {}
bob_var = 42 # iflabel {bob}
alice_bob_var = 69 # iflabel {alice,bob}

## Multiple assignment

# IF101: Success - Information flow from {} to {bob}, {alice,bob}
bob_var = alice_bob_var = 1

# IF101: Fail - Information flow from {} to {bob}, {alice,bob}
public_var = bob_var = alice_bob_var
