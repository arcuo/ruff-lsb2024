# ifprinciples {secret}

# iflabel fn (a: {secret}, b: {}) {secret}
def help(a,b):
  # Checking internal run of the function using arg labels
  secret_var = b # iflabel {secret}
  public_var = a # iflabel}

  return secret_var # OK but not if return was "public"

secret = 1 # iflabel {secret}
public = 2 # iflabel {}

# # Checking args
# help(secret, public) # OK
# help(public, secret) # Fail b has a public label

# Checking return
public = help(secret, public) # Fail
secret = help(secret, public) # Succeed 