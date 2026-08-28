#!/usr/bin/env bash
# Fake Minecraft server "java" used by the UI test suite.
# Accepts any arguments, prints the vanilla start banner, then echoes a
# console until it receives `stop` (or stdin closes).
echo 'Done (1.234s)! For help, type "help" or "?"'
while IFS= read -r line; do
    case "$line" in
        stop)
            echo 'Stopping the server'
            exit 0
            ;;
        list)
            echo 'There are 0 of a max of 20 players online: '
            ;;
        say*)
            echo "[Server] ${line#say}"
            ;;
        help|'')
            echo 'help: stop, list, say <msg>'
            ;;
        *)
            echo "Unknown command. Type \"help\" for help."
            ;;
    esac
done
