
# ifprinciples { alice, bob }


## var = opt

# Success
var1 = 69 # iflabel {alice,bob}

# Success
var2 = var1 # iflabel {alice, bob}

# Fail
var3 = var1 # iflabel {bob}

# Fail
var4 = var3 # iflabel {}



