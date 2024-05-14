# iflabel fn ({secret}, {public}) {secret}
def help(a,b):
  # Checking internal run of the function using arg labels
  some_outer_secret_value = a # OK
  some_outer_public_value = a # FAIL a is secret
  return public # OK
  return secret # OK but not if return was "public"

secret = 1 # iflabel {secret}
public = 2 # iflabel {secret}

# # Checking args
# help(secret, public) # OK
# help(public, secret) # Fail b has a public label

# Checking return
# secret = help(secret, public) # OK
public = help(secret, public) # Fail public cannot be assigned a secret return value from help
