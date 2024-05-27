
# ifprinciples { alice, bob }

public_var = 0 # iflabel {}
bob_var = 42 # iflabel {bob}
alice_bob_var = 69 # iflabel {alice,bob}

# ## var = expr 

# # IF101: Success - opt are considered public variables
alice_bob_var = 69 + 1

# # IF101: Fail Information flow from b to a
bob_var = alice_bob_var + 1

# # IF101: Success - Information flow from a to b
alice_bob_var = bob_var + 1
