# This script takes care of building your crate and packaging it for release

set -ex

main() {
    local src=$(pwd) \
          stage=

    case $TRAVIS_OS_NAME in
        linux)
            stage=$(mktemp -d)
            ;;
        osx)
            stage=$(mktemp -d -t tmp)
            ;;
    esac

    test -f Cargo.lock || cargo generate-lockfile

    # TODO Update this to build the artifacts that matter to you
    case $TRAVIS_OS_NAME in 
        osx)
            cross rustc --bin rust-http-server --target $TARGET --release -- -C lto
            ;;
        linux)
            cargo build --release --target $TARGET
            ;;
    esac

    # TODO Update this to package the right artifacts
    cp target/$TARGET/release/rust-http-server $stage/

    cd $stage
    tar czf $src/$CRATE_NAME-$TRAVIS_TAG-$TARGET.tar.gz *
    cd $src

    rm -rf $stage
}

main
