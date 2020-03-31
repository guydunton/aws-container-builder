#!/bin/bash -eu

WORKING_DIR=$HOME/.cbuilder

help() {
    echo ""
    echo "Usage: builder [OPTIONS] COMMAND"
    echo ""
    echo "Build containers remotely to save bandwidth"
    echo ""
    echo "Options:"
    echo "  -h, --help      Print the help text"
    echo ""
    echo "Commands:"
    echo "  bootstrap       Setup VM"
    echo "  ship            Ship directory to VM to build"
    echo "  cleanup         Remove the VM and connections to it"
    exit 0
}

ship_help() {
    echo ""
    echo "Usage: builder ship [OPTIONS] REGISTRY"
    echo ""
    echo "Send working directory to VM, build using docker and push to REGISTRY"
    echo ""
    echo "Options:"
    echo "  -h, --help      Print help text"
    exit 0
}

ship() {
    local REGISTRY
    if [[ $# -gt 1 ]]; then
        key="$2"

        case "$key" in
            -h|--help) # Ship help text
            ship_help
            ;;
            *) # Assume that anything else was the registry name
            REGISTRY="$2"
            shift
        esac
    else
        ship_help
    fi

    # Run the zip_and_ship script
    REGISTRY_NAME="$REGISTRY" "$WORKING_DIR"/scripts/zip_and_ship.sh
}

bootstrap_help() {
    echo ""
    echo "Usage: builder bootstrap [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -h, --help      Print help text"
    exit 0
}

bootstrap() {
    if [[ $# -gt 1 ]]; then
        key="$2"

        case "$key" in
            -h|--help) # Ship help text
            bootstrap_help
            ;;
            *)
            shift
            ;;
        esac
    fi

    "$WORKING_DIR"/scripts/bootstrap.sh
}

cleanup_help() {
    echo ""
    echo "Usage: builder cleanup [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -h, --help      Print help text"
    exit 0
}

cleanup() {
    if [[ $# -gt 1 ]]; then
        key="$2"

        case "$key" in
            -h|--help) # Ship help text
            cleanup_help
            ;;
            *)
        esac
    fi

    # Run the uninstall command
    "$WORKING_DIR"/scripts/uninstall.sh
}

if [[ $# -gt 0 ]]; then
    key="$1"

    case $key in
        ship) # Ship subcommand
        ship "$@"
        ;;
        bootstrap) # bootstrap subcommand
        bootstrap "$@"
        ;;
        cleanup) # cleanup subcommand
        cleanup "$@"
        ;;
        -h|--help) # root --help
        help
        ;;
        *) # Else
        help
    esac
else
    help
fi
