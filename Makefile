DOCKER_IMG_NAME ?= "ghcr.io/molguin92/agh-exporter-rs:latest"

.PHONY: build-docker
build-docker: Dockerfile
	docker build -t $(DOCKER_IMG_NAME) -f $< .

.PHONY: push-docker
push-docker: build-docker
	docker push $(DOCKER_IMG_NAME)