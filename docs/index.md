---
layout: default
title: BoxMux - YAML-driven Terminal UI Framework
---

<section class="feature-cards">
  <div class="container">
    <div class="section-header">
      <h1>BoxMux</h1>
      <p>YAML-driven terminal UI framework - Terminal applications and dashboards with data visualization, plugin system, and socket API.</p>

    <div class="hero-buttons">
      <a href="{{ "/user-guide" | relative_url }}" class="btn btn-primary">
        Get Started
      </a>
      <a href="https://github.com/jowharshamshiri/boxmux" class="btn btn-secondary" target="_blank">
        View on GitHub
      </a>
    </div>
    </div>
    
    <div class="section-header">
      <h2>Documentation</h2>
      <p>Set up your development environment and learn the fundamental concepts.</p>
    </div>
    
    <div class="cards-grid">
      <div class="feature-card setup-card">
        <h3>Setup and installation</h3>
        <p>Installation and basic configuration.</p>
        <a href="{{ "/user-guide" | relative_url }}" class="card-link">
          Get started
        </a>
      </div>
      
      <div class="feature-card api-card">
        <h3>API reference</h3>
        <p>Socket API documentation and function reference.</p>
        <a href="{{ "/api" | relative_url }}" class="card-link">
          API docs
        </a>
      </div>
      
      <div class="feature-card gallery-card">
        <h3>Configuration reference</h3>
        <p>YAML configuration options and examples.</p>
        <a href="{{ "/configuration" | relative_url }}" class="card-link">
          Browse configs
        </a>
      </div>
      
      <div class="feature-card">
        <h3>Data Visualization</h3>
        <p>Charts, tables, and data display features.</p>
        <a href="{{ "/data-visualization" | relative_url }}" class="card-link">
          Visualization guide
        </a>
      </div>
      
      <div class="feature-card">
        <h3>Plugin System</h3>
        <p>Dynamic component loading and plugin development.</p>
        <a href="{{ "/plugin-system" | relative_url }}" class="card-link">
          Plugin guide
        </a>
      </div>
      
      <div class="feature-card">
        <h3>PTY Features</h3>
        <p>Interactive terminal emulation and process management.</p>
        <a href="{{ "/pty-features" | relative_url }}" class="card-link">
          PTY guide
        </a>
      </div>
      
      <div class="feature-card">
        <h3>Advanced Features</h3>
        <p>Mouse support, hot keys, streaming output, and enhanced navigation.</p>
        <a href="{{ "/advanced-features" | relative_url }}" class="card-link">
          Advanced guide
        </a>
      </div>
    </div>
  </div>
</section>

<section class="quick-start">
  <div class="container">
    <div class="section-header">
      <h2>Quick Installation</h2>
    </div>

    <div class="install-steps">
      <div class="step">
        <div class="step-number">1</div>
        <h3>Install BoxMux</h3>
        <div class="code-block">
          cargo install boxmux
          <button class="copy-btn" onclick="copyToClipboard('cargo install boxmux')">
            <i class="fas fa-copy"></i>
          </button>
        </div>
      </div>
      
      <div class="step">
        <div class="step-number">2</div>
        <h3>Create your first interface</h3>
        <div class="code-block">
          # Create hello.yaml<br>
          boxmux hello.yaml
          <button class="copy-btn" onclick="copyToClipboard('boxmux hello.yaml')">
            <i class="fas fa-copy"></i>
          </button>
        </div>
      </div>
      
      <div class="step">
        <div class="step-number">3</div>
        <h3>Start building</h3>
        <div class="code-block">
          # Try example configs<br>
          boxmux examples/dashboard.yaml
          <button class="copy-btn" onclick="copyToClipboard('boxmux examples/dashboard.yaml')">
            <i class="fas fa-copy"></i>
          </button>
        </div>
      </div>
    </div>
  </div>
</section>

<section class="feature-cards">
  <div class="container">
    <div class="section-header">
      <h2>Key Features</h2>
      <p>Core functionality for building terminal interfaces.</p>
    </div>

    <div class="cards-grid">
      <div class="feature-card">
        <h3>Core Framework</h3>
        <p>YAML configuration system with multi-layout support, panel hierarchy, and real-time rendering.</p>
      </div>
      
      <div class="feature-card">
        <h3>UI Components</h3>
        <p>Flexible panel positioning, 16 ANSI colors, borders, text rendering, interactive menus, and focus management.</p>
      </div>
      
      <div class="feature-card">
        <h3>Scripting & Automation</h3>
        <p>Multi-threaded script execution, streaming output, PTY support, and output redirection.</p>
      </div>
      
      <div class="feature-card">
        <h3>Socket API</h3>
        <p>Unix socket server for remote control - update panels, switch layouts, manage refresh cycles.</p>
      </div>
      
      <div class="feature-card">
        <h3>Data Visualization</h3>
        <p>Unicode charts (bar/line/histogram), layout engine, table panels with CSV/JSON parsing, sorting, filtering.</p>
      </div>
      
      <div class="feature-card">
        <h3>Variable System</h3>
        <p>Hierarchical variable substitution with precedence: environment > child > parent > layout > app > default.</p>
      </div>
      
      <div class="feature-card">
        <h3>Plugin System</h3>
        <p>Dynamic component loading with security validation, manifest parsing, and access control.</p>
      </div>
      
      <div class="feature-card">
        <h3>Enhanced Features</h3>
        <p>Mouse clicks, hot keys (F1-F24), clipboard integration (Ctrl+C), scrolling, proportional scrollbars.</p>
      </div>
      
      <div class="feature-card">
        <h3>Performance & Quality</h3>
        <p>527/528 tests passing, performance benchmarking, cross-platform compatibility (macOS/Linux).</p>
      </div>
    </div>
  </div>
</section>

<script>
function copyToClipboard(text) {
  navigator.clipboard.writeText(text).then(() => {
    const btn = event.target.closest('.copy-btn');
    const icon = btn.querySelector('i');
    icon.className = 'fas fa-check';
    setTimeout(() => {
      icon.className = 'fas fa-copy';
    }, 2000);
  });
}
</script>
