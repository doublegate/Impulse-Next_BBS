# Dockerfile for Impulse BBS Automated Builds
FROM ubuntu:24.04

# Avoid interactive prompts during package installation
ENV DEBIAN_FRONTEND=noninteractive

# Install DOSBox and dependencies
RUN apt-get update && apt-get install -y \
    dosbox \
    xvfb \
    && rm -rf /var/lib/apt/lists/*

# Create directories for the build environment
RUN mkdir -p /impulse-build/source \
             /impulse-build/output \
             /impulse-build/include \
             /impulse-build/BP \
             /impulse-build/build

# Copy Borland Pascal from the repo
COPY BP/ /impulse-build/BP/

# Copy source files
COPY source/ /impulse-build/source/
COPY include/ /impulse-build/include/

# Create a Docker-optimized build script
COPY docker-build.sh /impulse-build/
RUN chmod +x /impulse-build/docker-build.sh

# Set working directory
WORKDIR /impulse-build

# Default command
CMD ["./docker-build.sh"]
