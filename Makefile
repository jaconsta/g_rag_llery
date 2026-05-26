SHELL := /bin/bash

.DEFAULT_GOAL := help

LOAD_ENV := set -a; source .env; set +a

.PHONY: help dev devcontainer build_devcontainer build format 

help: ## Show help
	@echo "Available commands:"
	@echo "  make help                       Show help"
	@echo "  make dev                        Run dev"
	@echo "  make devcontainer               SSH into devcontainer"
	@echo "  make build_devcontainer         Build/start devcontainer"
	@echo "  make build                      Build project"
	@echo "  make format                     Format code and run lint:fix"

dev: ## Run dev
	pnpm run dev

devcontainer: ## SSH into devcontainer
	# Will probably have to run this inside the container.
	# export PATH=$PATH:/home/vscode/.nix-profile/bin
	devpod ssh .

build_devcontainer: ## Build/start devcontainer
	devpod up . --provider docker --dotfiles git@github.com:jaconsta/nvim.git --ide none --open-ide false

build: ## Build project
	pnpm run build

format: ## Format code and run lint:fix
	pnpm run lint:fix
	pnpm run format
