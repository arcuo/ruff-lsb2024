# ifprincipals {secret}

# T_ASSIGN_IMPLICIT: max(label(value), pc) <= label(target)

secret_var = 0 # iflabel {secret}
public_var = 0 # iflabel {}

while secret_var < 10:
    public_var = 0 # Fail
    secret_var += 1 # Success
    while public_var < 10: # Nested while does not decrease the pc {secret}
        public_var += 1 # Fail


# TODO for loop