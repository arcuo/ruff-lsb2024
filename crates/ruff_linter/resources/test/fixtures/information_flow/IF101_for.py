
# ifprinciples { alice, bob }


public_var = 0 # iflabel {}
bob_var = 0 # iflabel {secret}
alice_bob_var = 0 # iflabel {alice,bob}


public_arr = [1,2,3]  # iflabel {}
bob_arr = [1,2,3] # iflabel {bob}
alice_bob_arr = [1,2,3] # iflabel {alice,bob}

for x in public_arr:
    public_var = x # Success
    bob_var = x # Success
    alice_bob_var = x # Success

for x in bob_arr:
    public_var = x # Fail
    bob_var = x # Success
    alice_bob_var = x # Success

for x in alice_bob_arr:
    public_var = x # Fail
    bob_var = x # Fail
    alice_bob_var = x # Success