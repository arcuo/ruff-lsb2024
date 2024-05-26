

# ifprinciples { alice, bob }

public_var = 0 # iflabel {}
bob_var = 42 # iflabel {bob}
alice_bob_var = 69 # iflabel {alice,bob}

## List assignment

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
