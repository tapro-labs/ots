default: build_prod

build_prod:
	docker build -f Dockerfile.backend build
	docker build -f Dockerfile.frontend build

dev-frontend:
	cd frontend && yarn && yarn dev

dev-backend:
	cd backend && make dev
