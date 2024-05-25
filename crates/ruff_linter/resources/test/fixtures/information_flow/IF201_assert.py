# ifprincipals {secret}

# T_ASSIGN_IMPLICIT: max(label(value), pc) <= label(target)

secret_var = 0 # iflabel {secret}
public_var = 0 # iflabel {}

assert secret_var > 0

public_var = 0 # Fail
public_var += 1 # Fail
public_var: int = 2 # Fail

secret_var = 1 # Success
secret_var += 1 # Success
secret_var: int = 2 # Success