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
    echo "  ship            Ship directory to VM to build"
    echo "  cleanup         Remove the VM and connections to it"
    echo "  connect         Connect to the VM"
    exit 0
}

ship_help() {
    echo ""
    echo "Usage: builder ship [OPTIONS] REGISTRY"
    echo ""
    echo "Send working directory to VM, build using docker and push to REGISTRY"
    echo ""
    echo "Options:"
    echo "  -h, --help          Print help text"
    echo "  -a, --account int   Push to remote account"
    exit 0
}

ship() {
    local REGISTRY
    local TARGET_ACCOUNT=""

    if [[ $# -gt 1 ]]; then
        while [[ $# -gt 0 ]]; do
            key="$1"

            echo "$key"

            case "$key" in
                -h|--help) # Ship help text
                ship_help
                ;;
                -a|--account) # Add remote account option
                TARGET_ACCOUNT="$2"
                shift
                shift
                ;;
                *) # Assume that anything else was the registry name
                REGISTRY="$1"
                shift
                break
                ;;
            esac
        done
    else
        ship_help
    fi

    # Run the zip_and_ship script
    REGISTRY_NAME="$REGISTRY" TARGET="$TARGET_ACCOUNT" "$WORKING_DIR"/scripts/zip_and_ship.sh "$@"
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
        shift
        ship "$@"
        ;;
        cleanup) # cleanup subcommand
        cleanup "$@"
        ;;
        connect) # connect subcommand
        "$WORKING_DIR"/scripts/connect.sh
        exit 0
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
