resource "aws_default_vpc" "vpc" {
  region = var.region
}

data "aws_subnet" "subnet" {
  region = var.region
  vpc_id = aws_default_vpc.vpc.id
  availability_zone_id = var.availability_zone_id
}

data "aws_security_group" "security_group" {
  region = var.region
  vpc_id = aws_default_vpc.vpc.id
  name = var.security_group_name
}
