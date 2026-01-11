# Palrun Fish Integration
# Add to your ~/.config/fish/config.fish: palrun init fish | source

function _palrun_search
    set -l output (palrun run 2>/dev/null)
    if test -n "$output"
        commandline -r $output
        commandline -f repaint
    end
end

bind \cp _palrun_search

# Alias for quick access
alias pal='palrun'
