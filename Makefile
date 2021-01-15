.PHONY: default build test clean build_release release_amd64_static release_armv7_static release install

default: release_static_arm

build:
	cargo build

test:
	cargo test

clean:
	cargo clean

build_release:
	cargo build --release

release_amd64_static:
	$(eval target := x86_64-unknown-linux-musl)
	cross build --release --target $(target)
	mkdir -p target/output
	cp target/$(target)/release/trackspeedtest target/output/
	docker buildx build -t giggio/speedtest:amd64 --platform linux/amd64 --build-arg PLATFORM=x86_64 .
	docker push giggio/speedtest:amd64

release_armv7_static:
	$(eval target := armv7-unknown-linux-musleabihf)
	cross build --release --target $(target)
	mkdir -p target/output
	cp target/$(target)/release/trackspeedtest target/output/
	docker buildx build -t giggio/speedtest:arm32v7 --platform linux/arm/v7 --build-arg PLATFORM=armhf .
	docker push giggio/speedtest:arm32v7

# release: release_amd64_static release_armv7_static
release:
	docker manifest create giggio/speedtest:latest \
		--amend giggio/speedtest:amd64 \
		--amend giggio/speedtest:arm32v7
	# docker manifest push giggio/speedtest:latest

install:
	echo todo
