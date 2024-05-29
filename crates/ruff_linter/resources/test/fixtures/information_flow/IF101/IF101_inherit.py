# ifprincipals {alice, bob}

a = 1 # iflabel {alice}
b = 2 # iflabel {bob}

b2 = b + 1 # should have Bob's label

a = b2 # Fail

c = a + b # Should have {alice, bob}

a = c # Fail
b = c # Fail

d = [a, b] # Should have {alice, bob}

c = d # Success
a = d # Fail