# Create an IAM role for github actions to assume
# https://github.com/aws-actions/configure-aws-credentials

data "aws_caller_identity" "id" {}

data "aws_iam_policy_document" "github_oidc_trust_policy" {
  statement {
    effect = "Allow"
    principals {
      type        = "Federated"
      identifiers = ["arn:aws:iam::${data.aws_caller_identity.id.account_id}:oidc-provider/token.actions.githubusercontent.com"]
    }
    actions = ["sts:AssumeRoleWithWebIdentity"]
    condition {
      test     = "StringEquals"
      variable = "token.actions.githubusercontent.com:aud"
      values   = ["sts.amazonaws.com"]
    }
    condition {
      test     = "StringEquals"
      variable = "token.actions.githubusercontent.com:sub"
      values   = ["repo:${var.github_path}:ref:refs/heads/master"]
    }
  }
}

resource "aws_iam_role" "github" {
  assume_role_policy = data.aws_iam_policy_document.github_oidc_trust_policy.json
  name = "${local.application_name}_github_role"
}
