# These values have to be used when creating or running the GitHub Action
output "aws_region" {
  value = var.region
}

output "aws_ecr_repo_url" {
  value = aws_ecr_repository.main.repository_url
}

output "aws_role" {
  description = "IAM Role that can be assumed by the GitHub Action"
  value       = aws_iam_role.github.arn
}
