# ifprincipals {secret}

# T_ASSIGN_IMPLICIT: max(label(value), pc) <= label(target)

secret_var = 0 # iflabel {secret}
secret_arr = [1,2,3] # iflabel {secret}
public_var = 0 # iflabel {}

for s in secret_var: # PC = {secret}
    public_var = 0 # Fail
    public_var += 1 # Fail
    public_var: int = 0 # Fail

    secret_var = 0 # Success
    secret_var += 1 # Success
    secret_var: int = 0 # Success

