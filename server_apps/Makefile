SHELL := /bin/bash

# NOTE: Figure out how to have a single Makefile in the project root
.DEFAULT_GOAL := help

LOAD_ENV := set -a; source .env; set +a
DATABASE_URL := postgres://crm_user:password@host.docker.internal:6543/rag_gallery_mfour

.PHONY: help dev build format 

help: ## Show help
	@echo "Available commands:"
	@echo "  make help                       Show help"
	@echo "  make build                      Build project"
	@echo "  make format                     Format code and run lint:fix"

migrate:
	sqlx migrate run --source ./db_storage/src/migrations/

db_new:
	sqlx database create

