#!/nix/store/lp3ginchcanhcj4dgw6yzdgv8bgdkm1v-bash-5.2p26/bin/sh

# This script publishes the package to npm.js, first perfoming validity checks.
# This script can be used locally or in CI safely.
# It assumes the package has already been installed, built, and tested.

set -e

cd "$(dirname "$0")"
cd ..

# Make sure everything is valid.
. scripts/check.sh

# Finally, publish the package. We assume it has been built
pnpm publish --non-interactive
