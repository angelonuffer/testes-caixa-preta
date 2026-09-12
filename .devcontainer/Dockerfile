FROM mcr.microsoft.com/devcontainers/rust:1-bookworm

# Instalar Chromium (para testes de navegação), Node.js e npm (para o servidor de testes) e utilitários
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get install -y --no-install-recommends chromium nodejs npm xz-utils sudo curl ca-certificates \
    && if [ ! -f /usr/bin/chromium-browser ] && [ -f /usr/bin/chromium ]; then \
    ln -s /usr/bin/chromium /usr/bin/chromium-browser; \
    fi \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# Instalar Nix em modo single-user para o usuário vscode com suporte a Flakes
RUN mkdir -p /nix && chown -R vscode:vscode /nix \
    && su vscode -c "curl -L https://nixos.org/nix/install | sh -s -- --no-daemon" \
    && mkdir -p /home/vscode/.config/nix \
    && echo "experimental-features = nix-command flakes" > /home/vscode/.config/nix/nix.conf \
    && chown -R vscode:vscode /home/vscode/.config

ENV PATH="/home/vscode/.nix-profile/bin:${PATH}"
