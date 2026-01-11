# Palrun Bash Integration
# Add to your ~/.bashrc: eval "$(palrun init bash)"

_palrun_search() {
    local output
    output=$(palrun run 2>/dev/null)
    if [[ -n "$output" ]]; then
        READLINE_LINE="$output"
        READLINE_POINT=${#output}
    fi
}

# Bind Ctrl+P to palrun
if [[ $- == *i* ]]; then
    bind -x '"\C-p": _palrun_search'
fi

# Alias for quick access
alias pal='palrun'
