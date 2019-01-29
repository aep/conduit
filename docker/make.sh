set -ex
(
    cd ..
    cargo build --release --target x86_64-unknown-linux-musl
    cp target/x86_64-unknown-linux-musl/release/conduit-scn docker
)

cp carrier.toml.tpl carrier.toml
CARRIER_CONFIG_FILE=$PWD/carrier.toml carrier setup

docker build . -t 531572303926.dkr.ecr.eu-west-1.amazonaws.com/superscale/conduit:$(git describe --tags)
docker push 531572303926.dkr.ecr.eu-west-1.amazonaws.com/superscale/conduit:$(git describe --tags)






