set -e

SCRIPT_DIR="$(cd "$(dirname "$0" )" && pwd)"
CCTRACE="$SCRIPT_DIR/../target/debug/cctrace"
CCEQ="$SCRIPT_DIR/../target/debug/cceq"
WORK_DIR=`mktemp -d` && cd ${WORK_DIR}

# deletes the temp directory
function cleanup {
  rm -rf "$WORK_DIR"
  echo "Deleted temp working directory $WORK_DIR"
}

# register cleanup function to be called on the EXIT signal
trap cleanup EXIT