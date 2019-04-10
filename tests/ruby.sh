#!/usr/bin/env bash

. "$(dirname "$0")/common.sh"

echo "downloading and extracting ruby"
curl -s https://cache.ruby-lang.org/pub/ruby/2.6/ruby-2.6.2.tar.gz | tar xz

RUBY_HOME="$WORK_DIR/ruby-2.6.2"

# TODO: test all of ruby, not just `ruby` target

cd ${RUBY_HOME}
./configure --quiet --disable-install-doc
intercept-build make ruby -j${NUM_PROCS}
mv compile_commands.json compile_commands.intercept
make clean
${RSTRACE} make ruby -j${NUM_PROCS}
mv compile_commands.json compile_commands.rstrace
cp compile_commands.* ${SCRIPT_DIR}
${CCEQ} compile_commands.intercept compile_commands.rstrace

echo -e "${GREEN}PASS${NC}"