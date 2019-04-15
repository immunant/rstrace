set -e

RED='\033[0;31m'
GREEN='\033[1;32m'
DGRAY='\033[1;30m'
NC='\033[0m' # No Color

NUM_PROCS=`nproc --all`

RUST_BACKTRACE=1

SCRIPT_DIR="$(cd "$(dirname "$0" )" && pwd)"
RSTRACE="$SCRIPT_DIR/../target/debug/rstrace"
CCEQ="$SCRIPT_DIR/../target/debug/cceq"
WORK_DIR=`mktemp -d` && cd ${WORK_DIR}

# deletes the temp directory
function cleanup {
  rm -rf "$WORK_DIR"
  echo -e "${DGRAY}Deleted temp working directory ${WORK_DIR}${NC}"
}

# register cleanup function to be called on the EXIT signal
trap cleanup EXIT