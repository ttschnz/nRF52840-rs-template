# build first
nix-shell --run "cargo build --release"

# convert to uf2
nix-shell --run "uf2conv target/thumbv7em-none-eabihf/release/nrf --base 0x27000 --family 0xADA52840 --output nrf.uf2"

# wait for mount
echo "waiting for mount"
until [ -d /run/media/tim/XIAO-SENSE/ ]; do sleep 1; done

# move to mount
mv nrf.uf2 /run/media/tim/XIAO-SENSE/
echo "done"