
# ifprinciples { alice, bob }

public_var = 0 # iflabel {}
bob_var = 42 # iflabel {bob}
alice_var = 33 # iflabel {alice}
alice_bob_var = 69 # iflabel {alice,bob}

## var = var

# Success
alice_bob_var = bob_var

# Fail
bob_var = alice_bob_var
# Fail
public_var = alice_bob_var

# Fail
public_var = bob_var

# Fail
alice_var = bob_var

# Success
alice_bob_var = public_var
# Success
bob_var = public_var