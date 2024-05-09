# ifprincipals {secret}

secret_var = 42 # iflabel {secret}
public_var = 0 # iflabel {}

if secret_var > 0:
    public_var = 1 # Fail - IF201 implicit information flow
    public_var += 1 # Fail - IF201 implicit information flow
    secret_var = 0 # Success - IF201 same label
else:
    public_var = 0 # Fail - IF201 implicit information flow
    public_var += 1 # Fail - IF201 implicit information flow
    secret_var = 0 # Success - IF201 same label


public_var = 1 # Success - IF201 no implicit information flow

# TODO: Q - Does this statement affect the information flow above?

# TODO: while?
# TODO: nested if?