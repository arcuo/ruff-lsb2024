
# ifprinciples { alice, bob }

public_var = 0 # iflabel {}
bob_var = 42 # iflabel {bob}
alice_var = 33 # iflabel {alice}
alice_bob_var = 69 # iflabel {alice,bob}

## Confidentality

# iflabel fn (a: {alice}) {}
def alice_fn(a):
    return alice_var # IF202

alice_var = alice_bob_var # IF101
bob_var = alice_bob_var # IF101

alice_fn(alice_bob_var) # IF203: Fail

if bob_var == 42:
    public_var = 69 # IF201

## Integrity

# iflabel fn (b: {bob}) {alice, bob}
def bob_fn(b):
    return bob_var # IF202

alice_bob_var = alice_var # IF101
alice_bob_var = bob_var # IF101

bob_fn(public_var) # IF203: Fail

if bob_var == 42:
    alice_bob_var = 69 # IF201


## Both

alice_var = bob_var # IF101
public_fn(bob_var) # IF203
