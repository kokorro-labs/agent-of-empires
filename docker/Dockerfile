# aoe-sandbox: Docker image for Agent of Empires sandbox sessions
# This image provides Claude Code, OpenCode, and OpenAI Codex in an isolated environment

FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive

# Install base packages and Node.js (required for Codex CLI)
RUN apt-get update && apt-get install -y \
    curl \
    git \
    build-essential \
    ca-certificates \
    gnupg \
    openssh-client \
    ripgrep \
    fzf \
    && rm -rf /var/lib/apt/lists/*

# Install Node.js LTS (required for OpenAI Codex CLI)
RUN curl -fsSL https://deb.nodesource.com/setup_lts.x | bash - \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/*

# Install OpenAI Codex CLI globally (as root, before switching to sandbox user)
RUN npm install -g @openai/codex

# Create non-root user for running Claude (required for --dangerously-skip-permissions)
# Use UID 1000 to match typical host user UID for proper file permissions on mounted volumes
RUN userdel -r ubuntu 2>/dev/null || true && \
    groupadd -g 1000 sandbox && \
    useradd -m -s /bin/bash -u 1000 -g 1000 sandbox

# Switch to sandbox user for tool installation
USER sandbox
WORKDIR /home/sandbox

# Install Claude Code CLI (official installer)
RUN curl -fsSL https://claude.ai/install.sh | bash
ENV PATH="/home/sandbox/.local/bin:${PATH}"

# Install OpenCode
RUN curl -fsSL https://opencode.ai/install | bash
ENV PATH="/home/sandbox/.opencode/bin:${PATH}"

# Create directories for credential mounts
RUN mkdir -p /home/sandbox/.claude \
    /home/sandbox/.config/opencode \
    /home/sandbox/.local/share/opencode \
    /home/sandbox/.openai \
    /home/sandbox/.ssh

# Set working directory
WORKDIR /workspace

# Default command - keep container running for docker exec
CMD ["sleep", "infinity"]
