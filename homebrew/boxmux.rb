class Boxmux < Formula
  desc "YAML-driven terminal UI framework for rich, interactive CLI applications"
  homepage "https://boxmux.com"
  url "https://github.com/jowharshamshiri/boxmux/archive/v0.1.0.tar.gz"
  sha256 "PLACEHOLDER_SHA256"
  license "MIT"
  head "https://github.com/jowharshamshiri/boxmux.git", branch: "main"

  depends_on "rust" => :build

  def install
    # Set environment variables for better optimization
    ENV["CARGO_TARGET_DIR"] = buildpath/"target"
    ENV["RUSTFLAGS"] = "-C target-cpu=native"
    
    # Build the project
    system "cargo", "install", "--root", prefix, "--path", "."
    
    # Install shell completions (if available)
    if File.exist?("completions")
      bash_completion.install Dir["completions/boxmux.bash"]
      zsh_completion.install Dir["completions/_boxmux"]
      fish_completion.install Dir["completions/boxmux.fish"]
    end

    # Install man page if available
    if File.exist?("docs/boxmux.1")
      man1.install "docs/boxmux.1"
    end
  end

  test do
    # Test that the binary runs and shows version
    assert_match "boxmux", shell_output("#{bin}/boxmux --version")
    
    # Test that help command works
    assert_match "USAGE:", shell_output("#{bin}/boxmux --help")
    
    # Test YAML configuration validation (create minimal test file)
    (testpath/"test.yaml").write <<~EOS
      app:
        frame_delay_ms: 16
      layouts:
        - name: test
          children:
            - id: test_box
              box_type: Content
              content: "Hello, World!"
              bounds:
                x: 0
                y: 0
                width: 100
                height: 100
    EOS
    
    # Test configuration validation
    system "#{bin}/boxmux", "validate", "test.yaml"
    
    # Cleanup
    rm "test.yaml"
  end
end