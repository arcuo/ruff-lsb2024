# ifprincipals {secret}

# T_ASSIGN_IMPLICIT: max(label(value), pc) <= label(target)

secret_var = 0 # iflabel {secret}
public_var = 0 # iflabel {}

if secret_var > 0: # PC = {secret}
    public_var = 0 # Fail
    public_var += 1 # Fail
    public_var: int = 2 # Fail

    secret_var = 0 # Success
    secret_var += 1 # Success
    secret_var: int = 2 # Success

    if public_var > 0: # Nested if does not decrease the pc {secret}
        public_var = 0 # Fail
        public_var += 1 # Fail
        public_var: int = 2 # Fail

elif public_var > 0: # Still in the same block i.e. the initial block sets the pc {secret}
    public_var = 0 # Fail
    public_var += 1 # Fail
    public_var: int = 2 # Fail

else: # Still in the same block i.e. the initial test sets the pc {secret}
    public_var = 0 # Fail
    public_var += 1 # Fail
    public_var: int = 2 # Fail

# PC should be reset after block

# TODO: Q - Does these statements below affect the information flow above?
public_var = 0 # Success
public_var: int = 2 # Success

# TODO: Q - This one definitely does not.
public_var += 1 # Success 

if public_var == 0:
    secret_var = 0 # Success
    secret_var += 1 # Success
    secret_var: int = 2 # Success

    public_var = 0 # Success
    public_var += 1 # Success
    public_var: int = 2 # Success

    if secret_var == 0: # Nested if increases the pc {secret}
        public_var = 0 # Fail
        public_var += 1 # Fail
        public_var: int = 2 # Fail

        secret_var = 0 # Success
        secret_var += 1 # Success
        secret_var: int = 2 # Success