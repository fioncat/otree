# example-basic.hcl
# Basic variable definitions
variable "environment" {
  description = "Deployment environment"
  type        = string
  default     = "development"
}

variable "instance_count" {
  description = "Number of instances"
  type        = number
  default     = 2
}

# Resource definitions
resource "aws_instance" "web" {
  count         = var.instance_count
  ami           = "ami-0c55b159cbfafe1d0"
  instance_type = "t2.micro"

  tags = {
    Name        = "web-server-${count.index}"
    Environment = var.environment
    Project     = "demo"
  }

  vpc_security_group_ids = [aws_security_group.web.id]
}

# Security group
resource "aws_security_group" "web" {
  name_prefix = "web-sg"
  
  ingress {
    from_port   = 80
    to_port     = 80
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  ingress {
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}
