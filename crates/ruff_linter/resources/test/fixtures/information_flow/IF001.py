
# ifprinciples { alice, bob }


a = 69 # iflabel {alice}
b = 42 # iflabel {bob}

# IF001: Fail - Information flow from a to b
a = b

# IF001: Success Information flow from b to a
b = a

