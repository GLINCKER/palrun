# Homebrew formula for Palrun
# Install: brew install GLINCKER/palrun/palrun

class Palrun < Formula
  desc "AI command palette for your terminal"
  homepage "https://github.com/GLINCKER/palrun"
  version "0.1.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/GLINCKER/palrun/releases/download/v#{version}/palrun-v#{version}-aarch64-apple-darwin.tar.gz"
      # sha256 "TO_BE_FILLED_AFTER_RELEASE"
    end
    on_intel do
      url "https://github.com/GLINCKER/palrun/releases/download/v#{version}/palrun-v#{version}-x86_64-apple-darwin.tar.gz"
      # sha256 "TO_BE_FILLED_AFTER_RELEASE"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/GLINCKER/palrun/releases/download/v#{version}/palrun-v#{version}-aarch64-unknown-linux-gnu.tar.gz"
      # sha256 "TO_BE_FILLED_AFTER_RELEASE"
    end
    on_intel do
      url "https://github.com/GLINCKER/palrun/releases/download/v#{version}/palrun-v#{version}-x86_64-unknown-linux-gnu.tar.gz"
      # sha256 "TO_BE_FILLED_AFTER_RELEASE"
    end
  end

  def install
    bin.install "palrun"
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

      Run 'palrun --help' to see all available commands.
    EOS
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/palrun --version")
    assert_match "palrun", shell_output("#{bin}/pal --version")
  end
end
