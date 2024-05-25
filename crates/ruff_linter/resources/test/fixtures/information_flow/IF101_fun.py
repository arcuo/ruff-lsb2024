# ifprinciples {alice, bob}

# iflabel fn ({alice}, {bob}) {bob}
def help(a,b):
  # Checking internal run of the function using arg labels
  some_outer_secret_value = a # OK
  some_outer_public_value = a # FAIL a is secret
  return b # OK
  return secret # OK but not if return was "public"

secret = 1 # iflabel {alice}
public = 2 # iflabel {bob}

# # Checking args
# help(secret, public) # OK
# help(public, secret) # Fail b has a public label

# Checking return
# secret = help(secret, public) # OK
secret = help(secret, public) # Fail public cannot be assigned a secret return value from help