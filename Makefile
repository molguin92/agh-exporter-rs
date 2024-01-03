DOCKER_IMG_NAME ?= "ghcr.io/molguin92/agh-exporter-rs"

.PHONY: docker-latest
docker-latest: Dockerfile
	docker buildx build --platform linux/amd64,linux/arm64 -t $(DOCKER_IMG_NAME):latest -f $< --push .

.PHONY: docker-version
docker-version-%: Dockerfile
	docker buildx build --platform linux/amd64,linux/arm64 -t $(DOCKER_IMG_NAME):$* -t $(DOCKER_IMG_NAME):latest -f $< --push .


.PHONY: release-bump-%
release-bump-%:
	cargo release $* --execute --no-confirm --no-push --no-publish
	TAG := $(shell git tag --points-at HEAD)
	$(MAKE) docker-version-$(TAG)
