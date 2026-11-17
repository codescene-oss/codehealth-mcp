# typed: false
# frozen_string_literal: true

class CsMcp < Formula
  desc "MCP Server exposing Code Health analysis as AI-friendly tools"
  homepage "https://github.com/codescene-oss/codehealth-mcp"
  version "1.5.22"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/codescene-oss/codehealth-mcp/releases/download/MCP-#{version}/cs-mcp-macos-aarch64.zip"
      sha256 "07121d8ad9451fd5d3c632c3f1fe4174942459d8333a49bc544fbafb35f6835f"

      define_method(:install) do
        bin.install "cs-mcp-macos-aarch64" => "cs-mcp"
      end
    end

    on_intel do
      url "https://github.com/codescene-oss/codehealth-mcp/releases/download/MCP-#{version}/cs-mcp-macos-amd64.zip"
      sha256 "d4b4384786dba0bd466b5e512375a4a5b23da1079ae6b9b6e77ecccbb1571833"

      define_method(:install) do
        bin.install "cs-mcp-macos-amd64" => "cs-mcp"
      end
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/codescene-oss/codehealth-mcp/releases/download/MCP-#{version}/cs-mcp-linux-aarch64.zip"
      sha256 "b803787596bf623e9d7110f4c2cd9d9966d6d69010fd63428d7f5f192e4eb410"

      define_method(:install) do
        bin.install "cs-mcp-linux-aarch64" => "cs-mcp"
      end
    end

    on_intel do
      url "https://github.com/codescene-oss/codehealth-mcp/releases/download/MCP-#{version}/cs-mcp-linux-amd64.zip"
      sha256 "9b6209eaa602a9b3afe311dacdc3c4bfb10dbb274185add602fc08c805f47fdd"

      define_method(:install) do
        bin.install "cs-mcp-linux-amd64" => "cs-mcp"
      end
    end
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/cs-mcp --version 2>&1", 2)
  end
end
