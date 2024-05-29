# ifprincipals { server, user }

# iflabel fn (user: {server})
def log_db(user):
    # Log user access to the database
    print("User {} accessed the database".format(user))

# iflabel fn (userId: {}, userCPR: {user}) {server, user}
def on_request(userId, userCPR):
    # Log the user's request
    log_db(userCPR)