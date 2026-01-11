# Palrun Zsh Integration
# Add to your ~/.zshrc: eval "$(palrun init zsh)"

palrun-widget() {
    local output
    output=$(palrun run 2>/dev/null)
    if [[ -n "$output" ]]; then
        BUFFER="$output"
        CURSOR=${#output}
        zle reset-prompt
    fi
}

zle -N palrun-widget
bindkey '^P' palrun-widget

# Alias for quick access
alias pal='palrun'
