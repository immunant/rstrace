#!/usr/bin/env bash

. "$(dirname "$0")/common.sh"

echo -e "${GREEN}downloading and extracting lua{NC}"
curl -s http://www.lua.org/ftp/lua-5.3.5.tar.gz | tar xz

LUA_HOME="$WORK_DIR/lua-5.3.5"
NUM_PROCS=`nproc --all`

cd ${LUA_HOME}
intercept-build make linux -j${NUM_PROCS}
mv compile_commands.json compile_commands.intercept
make clean
${RSTRACE} make linux -j${NUM_PROCS}
mv compile_commands.json compile_commands.rstrace
${CCEQ} compile_commands.intercept compile_commands.rstrace

echo -e "${GREEN}PASS${NC}"