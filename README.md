# tfvarsauto

Just utility that creates variables definition and outputs definition from terraform code

## Installation

You need to have rust to build binary first. And run in the root folder of this repository

```
$ cargo build --release
$ cp target/release/tfvarsauto ${HOME}/bin/
```

How to install Rust you can read in official Rust documentation ;)

## Usage

```
$ ./tfvarsauto -h
Usage: tfvarsauto [-h] [-m main.tf] [-v vars.tf] [-o outputs.tf]

Creates variables definition and
outputs definition from terraform code

Options:
    -h, --help          Print the usage menu
    -m, --main main.tf  The name of file for input with main terraform code,
                        main.tf by default
    -v, --vars vars.tf  The name of file to output with variables definition,
                        vars.tf by default
    -o, --outputs outputs.tf
                        The name of file to output with outputs definition,
                        outputs.tf by default
```

## Example

Let's imagine, that you wrote some terraform module, and save it in file `main.tf`, like this:

### main.tf

```
//noinspection MissingProperty
provider "azurerm" {
  version         = "~> 1.44"
  subscription_id = data.vault_generic_secret.az_subscription_id.data["id"]
  tenant_id       = data.vault_generic_secret.az_tenant_id.data["id"]
  client_id       = data.vault_generic_secret.az_client_id.data["id"]
  client_secret   = data.vault_generic_secret.az_client_secret.data["id"]
}

terraform {
  backend "azurerm" {
    storage_account_name = "tfstorageaccount"
    container_name       = "tfstate"
    key                  = "project.terraform.tfstate"
  }
}

provider "random" {
  version = "~> 2.1"
}

data "vault_generic_secret" "az_subscription_id" {
  path = var.az_subscription_id
}

data "vault_generic_secret" "az_tenant_id" {
  path = var.az_tenant_id
}

data "vault_generic_secret" "az_client_id" {
  path = var.az_client_id
}

data "vault_generic_secret" "az_client_secret" {
  path = var.az_client_secret
}

data "vault_generic_secret" "ssh_public" {
  path = var.ssh_public
}

locals {
  common_tags = {
    "Environment"    = upper(var.env_type)
  }
}

module "rsg" {
  source                 = "../modules/global/resource_group"
  az_location            = var.az_location
  az_resource_group_name = lower("RG-${local.common_tags["Environment"]}")

  tags = local.common_tags
}


module "vnet" {
  source                 = "../modules/network/vnet"
  az_location            = module.rsg.az_resource_group_location
  az_resource_group_name = module.rsg.az_resource_group_name
  az_vnet_name           = "proj-${local.common_tags["Environment"]}"
  az_vnet_cidr           = var.az_vnet_cidr

  tags = local.common_tags
}

module "subnet" {
  source                 = "../modules/network/subnet"
  az_resource_group_name = module.rsg.az_resource_group_name
  az_subnet_name         = "proj-${local.common_tags["Environment"]}"
  az_subnets             = var.az_subnets
  az_vnet_name           = module.vnet.az_vnet_name
  az_vnet_cidr           = module.vnet.az_vnet_address_space
  service_endpoints      = var.az_svc_endpoints
  az_subnet_bits         = var.az_subnet_bits
}

resource "azurerm_firewall_network_rule_collection" "az_firewall_rules" {
  action              = "Allow"
  azure_firewall_name = var.az_firewall_name
  name                = "test-name"
  priority            = 100
  resource_group_name = module.rsg.az_resource_group_name
  rule {
    name                  = "test-TCP"
    source_addresses      = [element(module.subnet.az_subnet_cidr, 1)]
    destination_addresses = var.az_firewall_tcp.dest_addrs
    destination_ports     = var.az_firewall_tcp.dest_ports
    protocols             = ["TCP"]
  }
  rule {
    name                  = "test-UDP"
    source_addresses      = [element(module.subnet.az_subnet_cidr, 1)]
    destination_addresses = var.az_firewall_udp.dest_addrs
    destination_ports     = var.az_firewall_udp.dest_ports
    protocols             = ["UDP"]
  }
}

resource "azurerm_route_table" "az_route_default" {
  location            = module.rsg.az_resource_group_location
  resource_group_name = module.rsg.az_resource_group_name
  name                = "route_table"

  route {
    name                   = "default"
    address_prefix         = "0.0.0.0/0"
    next_hop_type          = "VirtualAppliance"
    next_hop_in_ip_address = var.private_ip_address
  }
  route {
    name           = "fw"
    address_prefix = "${var.ip_address}/32"
    next_hop_type  = "Internet"
  }

  tags = local.common_tags
}

resource "azurerm_subnet_route_table_association" "az_assign_route" {
  subnet_id      = element(module.subnet.az_subnet_id, 1)
  route_table_id = azurerm_route_table.az_route_default.id
}
```

Now you have to create `vars.tf` with all variables definition and `outputs.tf` with all outputs of the module. So, you can make it with the `tfvarauto` utility. Just run it and you will have:

### vars.tf

```
variable "az_client_id" {
  type = string
  default = null
  description = ""
}

variable "az_client_secret" {
  type = string
  default = null
  description = ""
}

variable "az_firewall_name" {
  type = string
  default = null
  description = ""
}

variable "az_location" {
  type = string
  default = null
  description = ""
}

variable "az_subnet_bits" {
  type = string
  default = null
  description = ""
}

variable "az_subnets" {
  type = string
  default = null
  description = ""
}

variable "az_subscription_id" {
  type = string
  default = null
  description = ""
}

variable "az_svc_endpoints" {
  type = string
  default = null
  description = ""
}

variable "az_tenant_id" {
  type = string
  default = null
  description = ""
}

variable "az_vnet_cidr" {
  type = string
  default = null
  description = ""
}

variable "env_type" {
  type = string
  default = null
  description = ""
}

variable "ssh_public" {
  type = string
  default = null
  description = ""
}
```
So, you won't forget to define any variable and all what you need is change types, default values and add description!

### outputs.tf

```
output "data_vault_generic_secret_az_client_id" {
  value = data.vault_generic_secret.az_client_id
  description = ""
}

output "data_vault_generic_secret_az_client_secret" {
  value = data.vault_generic_secret.az_client_secret
  description = ""
}

output "data_vault_generic_secret_az_subscription_id" {
  value = data.vault_generic_secret.az_subscription_id
  description = ""
}

output "data_vault_generic_secret_az_tenant_id" {
  value = data.vault_generic_secret.az_tenant_id
  description = ""
}

output "data_vault_generic_secret_ssh_public" {
  value = data.vault_generic_secret.ssh_public
  description = ""
}

output "locals_common_tags" {
  value = locals.common_tags
  description = ""
}

output "module_rsg" {
  value = module.rsg
  description = ""
}

output "module_subnet" {
  value = module.subnet
  description = ""
}

output "module_vnet" {
  value = module.vnet
  description = ""
}

output "resource_azurerm_firewall_network_rule_collection_az_firewall_rules" {
  value = azurerm_firewall_network_rule_collection.az_firewall_rules
  description = ""
}

output "resource_azurerm_route_table_az_route_default" {
  value = azurerm_route_table.az_route_default
  description = ""
}

output "resource_azurerm_subnet_route_table_association_az_assign_route" {
  value = azurerm_subnet_route_table_association.az_assign_route
  description = ""
}
```

The same situation - you can copy blocks to define corresponded outputs from resources and add values. But you already have the template.
