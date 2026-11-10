#!/bin/sh
# Static terminal content for focus-glass-spike.sh: a claude-code-like transcript.
printf '\033[1;35m> \033[0mclaude (%s)\n\n' "$1"
printf '\033[36m\342\217\272 Read\033[0m src/layout/tile.rs\n'
printf '\033[36m\342\217\272 Bash\033[0m(cargo test -p niri-config)\n'
printf '  \033[2mrunning 214 tests ... ok\033[0m\n\n'
printf '\033[1;32m\342\234\224\033[0m done in 3.2s\n\n'
printf '\033[1m\342\235\257\033[0m '
