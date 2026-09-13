FROM mcr.microsoft.com/devcontainers/rust:1-bookworm

# Instalar Chromium (para testes de navegação), Node.js e npm (para o servidor de testes) e utilitários
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get install -y --no-install-recommends chromium nodejs npm xz-utils sudo curl ca-certificates \
    && if [ ! -f /usr/bin/chromium-browser ] && [ -f /usr/bin/chromium ]; then \
    ln -s /usr/bin/chromium /usr/bin/chromium-browser; \
    fi \
    && apt-get clean && rm -rf /var/lib/apt/lists/*
