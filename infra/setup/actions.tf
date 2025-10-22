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
      test     = "StringLike"
      variable = "token.actions.githubusercontent.com:sub"
      values   = ["repo:${var.github_path}:ref:refs/heads/*"]
    }
  }
}

resource "aws_iam_role" "github" {
  assume_role_policy = data.aws_iam_policy_document.github_oidc_trust_policy.json
  name               = "${local.application_name}_github_role"
}

data "aws_iam_policy_document" "ecr_login_policy" {
  statement {
    sid    = "GetAuthorizationToken"
    effect = "Allow"
    actions = [
      "ecr:GetAuthorizationToken"
    ]
    resources = ["*"]
  }
}

resource "aws_iam_policy" "ecr_login_policy" {
  policy = data.aws_iam_policy_document.ecr_login_policy.json
  name   = "ecr_login_policy"
}

resource "aws_iam_role_policy_attachment" "ecr_login_policy" {
  role       = aws_iam_role.github.name
  policy_arn = aws_iam_policy.ecr_login_policy.arn
}


data "aws_iam_policy_document" "ecr_push_pull_policy" {
  statement {
    sid    = "AllowPushPull"
    effect = "Allow"
    actions = [
      "ecr:BatchGetImage",
      "ecr:BatchCheckLayerAvailability",
      "ecr:CompleteLayerUpload",
      "ecr:GetDownloadUrlForLayer",
      "ecr:InitiateLayerUpload",
      "ecr:PutImage",
      "ecr:UploadLayerPart"
    ]
    resources = [aws_ecr_repository.main.arn]
  }
}

resource "aws_iam_policy" "ecr_push_pull_policy" {
  policy = data.aws_iam_policy_document.ecr_push_pull_policy.json
  name   = "ecr_push_pull_policy"
}

resource "aws_iam_role_policy_attachment" "ecr_push_pull_policy" {
  role       = aws_iam_role.github.name
  policy_arn = aws_iam_policy.ecr_push_pull_policy.arn
}
