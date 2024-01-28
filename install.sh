# Install CURL
if ! command -v curl &>/dev/null; then
    if ! command -v apt &>/dev/null; then
        echo "curl is not installed, and cannot be installed by this script"
        echo "Please install it manually then return here"
        exit 1
    fi
    echo "Installing curl"
    sudo apt install curl
fi

# Install RUST
if ! command -v cargo &>/dev/null; then
    echo "Installing rust..."
    echo " - You should accept the default installation option"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
fi

# Install NGINX
if ! command -v nginx &>/dev/null; then
    # If we're on mac, apt won't be present
    if ! command -v apt &>/dev/null; then
        # We must be on mac, make sure homebrew is installed
        if ! command -v brew &>/dev/null; then
            echo "Installing homebrew"
            /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
        fi

        echo "Installing NGINX..."
        sudo brew install nginx
    else
        echo "Installing NGINX..."
        sudo apt install nginx
    fi
fi

# Make nginx run on startup, and make sure its running now
if ! command -v apt &>/dev/null; then
    echo "Ensuring NGINX will run at startup"
    brew services start nginx
    echo "Starting nginx (might already be running, don't worry if you get errors)"
    nginx
else
    echo "Ensuring NGINX will run at startup"
    sudo systemctl enable nginx
    echo "Starting nginx (might already be running, don't worry if you get errors)"
    nginx
fi
