output "ECR_repo_url" {
  value = aws_ecr_repository.main.repository_url
}

output "github_role" {
  description = "IAM Role that can be assumed by the GitHub Action"
  value       = aws_iam_role.github.arn
}
