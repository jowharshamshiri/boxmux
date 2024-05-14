setup_user() {
  brew_dir=$(brew --prefix xbash 2>/dev/null)

  if [ -f "$brew_dir/xbashrc" ]; then
    cp "$brew_dir/xbashrc" "$HOME/.xbashrc"
    mkdir -p "$HOME/.xbash"
    cp -r "$brew_dir/lib" "$HOME/.xbash/lib"
    cp -r "$brew_dir/packages" "$HOME/.xbash/packages"
    cp -r "$brew_dir/package_repos" "$HOME/.xbash/package_repos"
    cp -r "$brew_dir/version.txt" "$HOME/.xbash/version.txt"
  else
    echo "xbash is not installed"
  fi
}

source_xbashrc() {
  if [ -f "$HOME/.xbashrc" ]; then
    source "$HOME/.xbashrc"
  else
    setup_user
    source "$HOME/.xbashrc"
  fi
}

source_xbashrc
