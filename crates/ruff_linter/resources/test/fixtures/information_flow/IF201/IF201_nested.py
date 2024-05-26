# ifprincipals {secret}

# T_ASSIGN_IMPLICIT: max(label(value), pc) <= label(target)

secret_var = 0 # iflabel {secret}
public_var = 0 # iflabel {}

## Nested WHILE in IF

if secret_var > 0: # PC = {secret}
    public_var = 0 # Fail
    while public_var < 10: # PC = {secret}
        public_var += 1 # Fail

if public_var > 0: # PC = {}
    public_var = 0 # Success
    while secret_var < 10: # PC = {secret}
        public_var += 1 # Fail

## Nested IF in WHILE

while secret_var < 10: # PC = {secret}
    public_var = 0 # Fail
    secret_var += 1 # Success
    if public_var < 10: # PC = {secret}
        public_var += 1 # Fail

while public_var < 10:
    public_var += 1 # Fail
    if secret_var < 10: # PC = {secret}
        secret_var += 1 # Success

## WHILE/IF after ASSERT

assert secret_var > 0

public_var = 0 # Fail

if public_var == 0:
    public_var += 1 # Fail

while public_var < 10:
    public_var += 1 # Fail


# TODO for loop