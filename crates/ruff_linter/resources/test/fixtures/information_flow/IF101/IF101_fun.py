# ifprinciples {alice, bob}

# iflabel fn (a: {alice}, b: {bob}, public: {}) {alice}
def help(a,b, public):
  # Checking internal run of the function using arg labels

  # Succeed
  var1 = a # iflabel {alice}
  var1 = a # iflabel {alice, bob}

  # Fail
  var2 = a # iflabel {bob}
  var2 = a # iflabel {}

  # Succeed
  var3 = b # iflabel {bob}
  var3 = b # iflabel {alice, bob}

  # Fail
  var4 = b # iflabel {alice}
  var4 = b # iflabel {}


  return alice_var # OK but not if return was "public"

alice = 1 # iflabel {alice}
public = 2 # iflabel {}

# # Checking args
# help(alice, public) # OK
# help(public, alice) # Fail b has a public label

# Checking return
public = help(alice, public) # Fail
alice = help(alice, public) # Succeed 