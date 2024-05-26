# ifprinciples {alice, bob}

# iflabel fn (a: {alice}, b: {bob}, public: {}) {alice}
def help(a,b, public):
  # Checking internal run of the function using arg labels

  # Succeed
  var1 = a # iflabel {alice}
  var1 = a # iflabel {alice, bob}

  # Fail
  var2 = a # iflabel {}
  var2 = a # iflabel {bob}

  # Succeed
  var3 = b # iflabel {bob}
  var3 = b # iflabel {alice, bob}

  # Fail
  var4 = b # iflabel {}
  var4 = b # iflabel {alice}


  alice_return = 1 # iflabel {alice}
  public_return = 3 # iflabel {}
  bob_return = 2 # iflabel {bob}
  alice_bob_return = 4 # iflabel {alice, bob}


  return alice_return # Succeed
  return public_return # Succeed
  return bob_return # Fail
  return alice_bob_return # Fail

alice = 1 # iflabel {alice}
public = 2 # iflabel {}

# # Checking args
# help(alice, public) # OK
# help(public, alice) # Fail b has a public label

# Checking return
public = help(alice, public) # Fail
alice = help(alice, public) # Succeed 