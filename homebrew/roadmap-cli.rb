# Homebrew formula for roadmap-cli
# Place this in your homebrew-tap repo: Formula/roadmap-cli.rb
#
# Usage:
#   brew tap siovos/tap
#   brew install roadmap-cli

class RoadmapCli < Formula
  desc "CLI pour gérer les roadmaps projet avec phases, tâches et workflows"
  homepage "https://github.com/siovos/roadmap-cli"
  version "0.1.0"
  license "MIT"

  on_macos do
    on_intel do
      url "https://github.com/siovos/roadmap-cli/releases/download/v#{version}/roadmap-cli-darwin-x86_64.tar.gz"
      # sha256 "REPLACE_WITH_ACTUAL_SHA256"
    end

    on_arm do
      url "https://github.com/siovos/roadmap-cli/releases/download/v#{version}/roadmap-cli-darwin-arm64.tar.gz"
      # sha256 "REPLACE_WITH_ACTUAL_SHA256"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/siovos/roadmap-cli/releases/download/v#{version}/roadmap-cli-linux-x86_64.tar.gz"
      # sha256 "REPLACE_WITH_ACTUAL_SHA256"
    end
  end

  def install
    bin.install "roadmap-cli" => "roadmap"
  end

  test do
    system "#{bin}/roadmap", "--version"
  end
end
