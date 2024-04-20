
# ifauthorities = {
#     alice,
#     bob,
# }

# iflabel {alice, [bob]}
a = 69
# iflabel {bob, []}
b = 42

# IF001: Fail - Information flow from a to b
a = b

# IF001: Success Information flow from b to a
b = a

