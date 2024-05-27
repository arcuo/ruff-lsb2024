# ifprinciples {alice}

def public_log(p):
  pass

# iflabel fn (p: {})
def help(p):
  public_log(p) # public argument is fed to a public function

secret = "password" # iflabel {alice}

## Argument

help(secret) #  Fail
help("hello") # Succeed

## Keyword

help(p=secret) # Fail
help(p="hello") # Succeed