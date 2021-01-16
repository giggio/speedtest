.PHONY: default build test clean build_release build_amd64_static docker_build_amd64_static release_amd64_static build_armv7_static docker_build_armv7_static release_armv7_static release_with_docker_only release

amd64_target := x86_64-unknown-linux-musl
arm32v7_target := armv7-unknown-linux-musleabihf
default: release_static_arm

build:
	cargo build

test:
	cargo test

clean:
	cargo clean

build_release:
	cargo build --release

build_amd64_static:
	cross build --release --target $(amd64_target)

docker_build_amd64_static:
	mkdir -p target/output
	cp target/$(amd64_target)/release/trackspeedtest target/output/
	docker buildx build -t giggio/speedtest:amd64 --platform linux/amd64 --build-arg PLATFORM=x86_64 --push .

release_amd64_static: build_amd64_static docker_build_amd64_static

build_armv7_static:
	cross build --release --target $(arm32v7_target)

docker_build_armv7_static:
	mkdir -p target/output
	cp target/$(arm32v7_target)/release/trackspeedtest target/output/
	docker buildx build -t giggio/speedtest:arm32v7 --platform linux/arm/v7 --build-arg PLATFORM=armhf --push .

release_armv7_static: build_armv7_static docker_build_armv7_static

release_with_docker_only: docker_build_amd64_static docker_build_armv7_static
	DOCKER_CLI_EXPERIMENTAL=enabled docker manifest create giggio/speedtest:latest \
		--amend giggio/speedtest:amd64 \
		--amend giggio/speedtest:arm32v7
	DOCKER_CLI_EXPERIMENTAL=enabled docker manifest push giggio/speedtest:latest

release: release_amd64_static release_armv7_static release_with_docker_only
