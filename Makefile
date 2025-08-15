.PHONY: install lint test build typecheck format ci perf e2e up down logs chaos clean

# Install dependencies
install:
	npm ci

# Run linting
lint:
	npm run lint

# Run tests with coverage
test:
	npm run test

# Type checking
typecheck:
	npm run typecheck

# Format code
format:
	npm run format

# Build project
build:
	npm run build

# Full CI pipeline
ci: install typecheck lint test build

# Performance tests (requires Docker)
perf:
	docker compose -f docker-compose.production.yml up -d --build
	@echo "Waiting for services..."
	@sleep 30
	docker run --rm -i --network host -e AI_URL=http://localhost:8081 -v "$(PWD)/perf:/perf" grafana/k6 run /perf/ai-load.js
	docker run --rm -i --network host -e UI_URL=http://localhost:3000 -v "$(PWD)/perf:/perf" grafana/k6 run /perf/ui-smoke.js
	docker compose -f docker-compose.production.yml down -v

# E2E tests
e2e:
	docker compose -f docker-compose.production.yml up -d --build
	npm run e2e
	docker compose -f docker-compose.production.yml down -v

# Chaos testing
chaos:
	docker compose -f docker-compose.production.yml up -d --build
	@echo "Starting chaos test..."
	bash scripts/chaos-kill-one.sh
	docker run --rm -i --network host -e AI_URL=http://localhost:8081 -v "$(PWD)/perf:/perf" grafana/k6 run /perf/ai-load.js
	docker compose -f docker-compose.production.yml down -v

# Start development stack
up:
	docker compose -f docker-compose.production.yml up -d --build

# Stop all containers
down:
	docker compose -f docker-compose.production.yml down -v

# View logs
logs:
	docker compose -f docker-compose.production.yml logs -f

# Clean everything
clean:
	docker compose -f docker-compose.production.yml down -v --remove-orphans
	docker system prune -f
	npm clean-install --prefer-offline

# Health check
health:
	@echo "=== SYSTEM HEALTH CHECK ==="
	@echo "UI Health:"
	@curl -f http://localhost:3000/api/health 2>/dev/null | jq . || echo "UI not responding"
	@echo "AI Health:"  
	@curl -f http://localhost:8081/health 2>/dev/null | jq . || echo "AI not responding"
	@echo "Backend Health:"
	@curl -f http://localhost:8080/health 2>/dev/null | jq . || echo "Backend not responding"

# Full production test
production-test: clean up
	@echo "Waiting for full stack startup..."
	@sleep 60
	$(MAKE) health
	$(MAKE) perf
	$(MAKE) e2e
	$(MAKE) chaos
	$(MAKE) down
	@echo "✅ FULL PRODUCTION TESTING COMPLETE"
