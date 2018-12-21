python scripts/skull_setup.py assets/skulls.svg assets/

python scripts/feature_setup.py assets/nose.svg assets/
python scripts/feature_setup.py assets/ears.svg assets/
python scripts/feature_setup.py assets/eyes.svg assets/
python scripts/feature_setup.py assets/hair.svg assets/
python scripts/feature_setup.py assets/mouth.svg assets/

python scripts/feature_setup.py assets/skull_cap.svg assets/
python scripts/feature_setup.py assets/cheek_bones.svg assets/
python scripts/feature_setup.py assets/mandible.svg assets/

RUST_BACKTRACE=1 cargo run > /tmp/test.svg
