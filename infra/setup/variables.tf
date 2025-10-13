# NOTE: These are the variables that you must set!
# Also take a look at network.tf if you want to use a different VPC
# than the default one in a var.region.
variable "region" {
  description = "AWS Region to use"
  type        = string
}

variable "availability_zone_id" {
  description = "Availability zone id in region to use"
  type        = string
}

variable "security_group_name" {
  description = "Security group's name to use"
  type        = string
}

variable "discord_token" {
  description = "Discord bot token"
  type        = string
  sensitive   = true
}

variable "genius_token" {
  description = "Genius API token"
  type        = string
  sensitive   = true
}


locals {
  application_name        = "genius-discord-bot"
  application_description = "Discord bot to create genius-like cards"
}
