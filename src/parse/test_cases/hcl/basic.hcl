variable {
  environment {
    description = "Deployment environment"
    type = "string"
    default = "development"
  }
  instance_count {
    description = "Number of instances"
    type = "number"
    default = 2
  }
  enabled {
    description = "Whether to enable"
    type = "bool"
    default = true
  }
}
config {
  name = "test-app"
  version = "1.0.0"
  port = 8080
  debug = false
}
