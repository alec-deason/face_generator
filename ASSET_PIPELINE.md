# Setup

## Generate Assets

### Skulls
* python scripts/skull_setup.py assets/skulls.svg assets/

### Features
* python scripts/feature_setup.py assets/nose.svg assets/
* python scripts/feature_setup.py assets/ears.svg assets/
* python scripts/feature_setup.py assets/mouth.svg assets/

## Create Test Output
RUST_BACKTRACE=1 cargo run > /tmp/test.svg
