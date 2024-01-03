DOCKER_IMG_NAME ?= "ghcr.io/molguin92/agh-exporter-rs"

.PHONY: docker-latest
docker-latest: Dockerfile
	docker buildx build --platform linux/amd64,linux/arm64 -t $(DOCKER_IMG_NAME):latest -f $< --push .


.PHONY: release-bump-%
release-bump-%: Dockerfile.from_crates_io
	cargo release $* --execute --no-confirm
	docker buildx build --platform linux/amd64,linux/arm64 -t $(DOCKER_IMG_NAME):latest -f $< --push .
