# Homebrew formula for Palrun
# To install locally: brew install --build-from-source ./scripts/palrun.rb
# To submit to homebrew-core, this file would need SHA256 checksums

class Palrun < Formula
  desc "AI command palette for your terminal"
  homepage "https://github.com/GLINCKER/palrun"
  url "https://github.com/GLINCKER/palrun/archive/refs/tags/v0.1.0.tar.gz"
  # sha256 "TO_BE_FILLED_AFTER_RELEASE"
  license "MIT"
  head "https://github.com/GLINCKER/palrun.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args

    # Install shell completions
    generate_completions_from_executable(bin/"palrun", "completions")

    # Create 'pal' alias
    bin.install_symlink "palrun" => "pal"
  end

  def caveats
    <<~EOS
      Palrun has been installed as both 'palrun' and 'pal'.

      To enable shell integration, add the following to your shell config:

      For bash (~/.bashrc):
        eval "$(palrun init bash)"

      For zsh (~/.zshrc):
        eval "$(palrun init zsh)"

      For fish (~/.config/fish/config.fish):
        palrun init fish | source
    EOS
  end

  test do
    assert_match "palrun", shell_output("#{bin}/palrun --version")
    assert_match "palrun", shell_output("#{bin}/pal --version")

    # Test that list command works (no commands in empty temp dir is expected)
    output = shell_output("#{bin}/palrun list 2>&1", 0)
    assert_match(/Total: \d+ commands/, output)
  end
end
