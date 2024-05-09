# ifprincipals {alice, bob}


## BAD

b = 0 # iflabel {charlie}
b2: int = 0 # iflabel {alice, charlie}

## GOOD

a1 = 0 # iflabel {bob}
a2 = 0 # iflabel {alice}

