# 1. Register a new application
provider "aws" {
  # NOTE: we first need to create an application to be able to get its tag
  # therefore this provider block is just for that, the "proper" provider will be
  # specified later, this
  alias  = "application"
  region = var.region
}

resource "aws_servicecatalogappregistry_application" "app" {
  provider    = aws.application
  name        = local.application_name
  description = local.application_description
}

provider "aws" {
  # NOTE: this is the "proper" provider
  region = var.region
  default_tags {
    tags = aws_servicecatalogappregistry_application.app.application_tag
  }
}

# 2. create an ECR repo
resource "aws_ecr_repository" "main" {
  name = "${local.application_name}_repo"
}

# 3. create cluster
resource "aws_ecs_cluster" "main" {
  name = "${local.application_name}_cluster"
}

# 4. register task definition
resource "aws_iam_policy" "create_log_group" {
  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect = "Allow"
      Action = [
        "logs:CreateLogGroup"
      ]
      Resource = "*"
    }]
  })
}

resource "aws_iam_role" "task_execution" {
  name = "ecsTaskExecutionRole"
  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Sid    = ""
      Effect = "Allow"
      Principal = {
        Service = "ecs-tasks.amazonaws.com"
      }
      Action = "sts:AssumeRole"
    }]
  })
}

data "aws_iam_policy" "AmazonECSTaskExecutionRolePolicy" {
  arn = "arn:aws:iam::aws:policy/service-role/AmazonECSTaskExecutionRolePolicy"
}

resource "aws_iam_role_policy_attachment" "create_log_group" {
  role       = aws_iam_role.task_execution.name
  policy_arn = aws_iam_policy.create_log_group.arn
}

resource "aws_iam_role_policy_attachment" "AmazonECSTaskExecutionRolePolicy" {
  role       = aws_iam_role.task_execution.name
  policy_arn = data.aws_iam_policy.AmazonECSTaskExecutionRolePolicy.arn
}

# NOTE: when this config is first applied this will fail as the ECR repo is empty
resource "aws_ecs_task_definition" "task" {
  family       = "${local.application_name}_task_definition"
  network_mode = "awsvpc"
  container_definitions = jsonencode([
    {
      name      = "${local.application_name}_container"
      image     = "${aws_ecr_repository.main.repository_url}:latest"
      essential = true
      logConfiguration = {
        logDriver = "awslogs"
        options = {
          awslogs-create-group  = "true"
          awslogs-group         = "awslogs-${local.application_name}"
          awslogs-region        = var.region
          awslogs-stream-prefix = "${local.application_name}"
        }
      }
      linuxParameters = {
        initProcessEnabled = true
      }
      environment = [
        {
          name  = "GENIUS_TOKEN"
          value = var.genius_token
        },
        {
          name  = "DISCORD_TOKEN"
          value = var.discord_token
        }
      ]
    }
  ])
  requires_compatibilities = [
    "FARGATE"
  ]
  execution_role_arn = aws_iam_role.task_execution.arn
  task_role_arn      = aws_iam_role.task_execution.arn
  cpu                = 256
  memory             = 512
}

# 5. create service
# TODO: creating a VPC endpoint to avoid assiging public IP
# https://docs.aws.amazon.com/vpc/latest/privatelink/create-interface-endpoint.html
# Use a VPC endpoint for Amazon ECR. VPC endpoints are powered by AWS PrivateLink,
# a technology that enables you to privately access Amazon ECR APIs through private IP addresses.
# For Amazon ECS tasks that use the Fargate launch type, the VPC endpoint enables the task to pull private images
# from Amazon ECR without assigning a public IP address to the task.

resource "aws_ecs_service" "service" {
  name                   = "${local.application_name}_service"
  cluster                = aws_ecs_cluster.main.id
  task_definition        = aws_ecs_task_definition.task.arn
  desired_count          = 1
  launch_type            = "FARGATE"
  depends_on             = []
  enable_execute_command = true
  network_configuration {
    subnets          = [data.aws_subnet.subnet.id]
    security_groups  = [data.aws_security_group.security_group.id]
    assign_public_ip = true
  }
}
