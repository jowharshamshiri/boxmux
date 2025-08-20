#!/bin/bash

# Script to serve the BoxMux documentation site locally
# Requires Jekyll to be installed: gem install jekyll bundler

echo "🚀 Starting BoxMux Documentation Server..."

# Check if Jekyll is installed
if ! command -v jekyll &> /dev/null; then
    echo "❌ Jekyll not found. Installing..."
    gem install jekyll bundler
fi

# Check if Gemfile exists, if not create one
if [ ! -f "Gemfile" ]; then
    echo "📦 Creating Gemfile..."
    cat > Gemfile << EOF
source "https://rubygems.org"

gem "jekyll", "~> 4.3"
gem "jekyll-sitemap"
gem "jekyll-feed"
gem "jekyll-seo-tag"

group :jekyll_plugins do
  gem "jekyll-sitemap"
  gem "jekyll-feed"
  gem "jekyll-seo-tag"
end
EOF
fi

# Install dependencies
echo "📦 Installing dependencies..."
bundle install

# Serve the site
echo "🌐 Serving site at http://localhost:4000"
echo "🔄 Press Ctrl+C to stop"
JEKYLL_ENV=development bundle exec jekyll serve --config _config.yml,_config_dev.yml --livereload --open-url --source .
