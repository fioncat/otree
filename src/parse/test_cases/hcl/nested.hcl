company {
  TechCorp {
    location = "NYC"
    department {
      engineering {
        active = true
        employee {
          alice {
            role = "developer"
            name = "Alice"
            age = 29
          }
          bob {
            role = "tester"
            name = "Bob"
            age = 32
          }
        }
      }
      hr {
        active = false
        employee {
          charlie {
            role = "manager"
            name = "Charlie"
            age = 40
          }
        }
      }
    }
  }
}
network {
  vpc {
    cidr_block = "10.0.0.0/16"
    subnets {
      cidr_block = "10.0.1.0/24"
      az = "us-east-1a"
      public = true
    }
    subnets {
      cidr_block = "10.0.2.0/24"
      az = "us-east-1b"
      public = false
    }
  }
}
