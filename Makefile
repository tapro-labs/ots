default: build_prod

build_prod:
	docker build -f Dockerfile.backend build
	docker build -f Dockerfile.frontend build

dev_frontend:
	cd frontend && yarn && yarn dev

dev_backend:
	cd backend && make dev

start_docker:
	docker-compose up -d

stop_docker:
	docker-compose down

generate_ssl:
	./server_ssl/generate-ssl.sh

see_docs:
	cd docs && mdbook serve --open -p 9050
