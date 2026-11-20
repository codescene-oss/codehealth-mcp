# typed: false
# frozen_string_literal: true

class CsMcp < Formula
  desc "MCP Server exposing Code Health analysis as AI-friendly tools"
  homepage "https://github.com/codescene-oss/codehealth-mcp"
  version "1.5.23"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/codescene-oss/codehealth-mcp/releases/download/MCP-#{version}/cs-mcp-macos-aarch64.zip"
      sha256 "ef81ed7dbbfb9f3acf0ca46d057a7f8765b622505bb7fd9046398d188ace3ae1"

      define_method(:install) do
        bin.install "cs-mcp-macos-aarch64" => "cs-mcp"
      end
    end

    on_intel do
      url "https://github.com/codescene-oss/codehealth-mcp/releases/download/MCP-#{version}/cs-mcp-macos-amd64.zip"
      sha256 "1aa1e8ffd3642dfac7eb0f0f931322b73681ffd3f43f8d8a1a11033242228033"

      define_method(:install) do
        bin.install "cs-mcp-macos-amd64" => "cs-mcp"
      end
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/codescene-oss/codehealth-mcp/releases/download/MCP-#{version}/cs-mcp-linux-aarch64.zip"
      sha256 "cd251e17ea7d52591a3ea5e6e47066cc251f6545ecccb66bb846754be272abd2"

      define_method(:install) do
        bin.install "cs-mcp-linux-aarch64" => "cs-mcp"
      end
    end

    on_intel do
      url "https://github.com/codescene-oss/codehealth-mcp/releases/download/MCP-#{version}/cs-mcp-linux-amd64.zip"
      sha256 "33fb30f28c324d9edc8473a2ac899e9dd1b267075f742e4372c962476a06116c"

      define_method(:install) do
        bin.install "cs-mcp-linux-amd64" => "cs-mcp"
      end
    end
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/cs-mcp --version 2>&1", 2)
  end
end
