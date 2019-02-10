# This script takes care of testing your crate

set -ex

main() {
    pushd psm

    cross build --target $TARGET
    cross build --target $TARGET --release

    if [ ! -z $DISABLE_TESTS ]; then
        return
    fi

    cross test --target $TARGET -- --nocapture --test-threads=1
    cross test --target $TARGET --release -- --nocapture --test-threads=1

    cross test --target $TARGET --examples -- --nocapture --test-threads=1
    cross test --target $TARGET --examples --release -- --nocapture --test-threads=1

    popd

    cross test --target $TARGET
    cross test --release --target $TARGET
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi
