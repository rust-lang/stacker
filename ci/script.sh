# This script takes care of testing your crate

set -ex

main() {
    cross build --target $TARGET
    cross build --target $TARGET --release

    if [ ! -z $DISABLE_TESTS ]; then
        return
    fi

    cross test --target $TARGET -- --nocapture
    cross test --target $TARGET --release -- --nocapture

    cross test --target $TARGET --examples -- --nocapture
    cross test --target $TARGET --examples --release -- --nocapture
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi
