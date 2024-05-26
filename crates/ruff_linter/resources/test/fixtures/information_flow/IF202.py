# ifprinciples {alice, bob}

# iflabel fn () {alice}
def fn1():
  alice_return = 1 # iflabel {alice}

  return alice_return # Succeed

# iflabel fn () {alice}
def fn2():
  public_return = 3 # iflabel {}

  return public_return # Succeed

# iflabel fn () {alice}
def fn3():
  bob_return = 2 # iflabel {bob}

  return bob_return # Fail

# iflabel fn () {alice}
def fn4():
  alice_bob_return = 4 # iflabel {alice, bob}

  return alice_bob_return # Fail