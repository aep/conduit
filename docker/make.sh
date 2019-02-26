set -ex
(
    cd ..
    cargo build --release
    cp target/release/conduit-scn docker
)



docker build . -t 531572303926.dkr.ecr.eu-west-1.amazonaws.com/superscale/conduit:$(git describe --tags)
docker push 531572303926.dkr.ecr.eu-west-1.amazonaws.com/superscale/conduit:$(git describe --tags)




git describe --tags

