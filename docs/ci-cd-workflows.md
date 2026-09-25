# CI/CD Workflows Documentation

**Version:** 1.0.0
**Last Updated:** 2025-11-26
**Status:** Production Ready

---

## Overview

Impulse-Next_BBS uses GitHub Actions for continuous integration and automated releases. This document describes the two main workflows:

1. **CI Workflow** (`.github/workflows/ci.yml`) - Continuous integration testing and quality checks
2. **Release Workflow** (`.github/workflows/release.yml`) - Automated release builds and distribution

---

## CI Workflow

**File:** `.github/workflows/ci.yml`
**Triggers:**
- Push to `main` or `develop` branches
- Pull requests to `main` or `develop` branches

### Workflow Structure

The CI pipeline consists of 8 jobs organized in a dependency graph for optimal performance:

```
lint ──┬─→ test ──┬─→ build
       │          ├─→ coverage
       │          └─→ benchmark
       ├─→ security
       └─→ msrv
              │
              └──→ ci-success (gate)
```

### Job Details

#### 1. **lint** (Fast-fail check)
- **Platform:** ubuntu-latest
- **Duration:** ~1 minute
- **Purpose:** Format and linting checks
- **Steps:**
  - Check code formatting with `cargo fmt`
  - Run clippy with strict warnings (`-D warnings`)
- **Caching:** Shared cache key "lint"

#### 2. **test** (Multi-platform)
- **Platform:** ubuntu-latest, windows-latest, macos-latest
- **Duration:** ~3-4 minutes per platform
- **Purpose:** Run test suite on all platforms
- **Steps:**
  - Run workspace tests with `--all-features --verbose`
  - Run documentation tests
- **Caching:** Per-OS cache key "test-{os}"
- **Dependencies:** Requires `lint` to pass

#### 3. **build** (Multi-platform)
- **Platform:** ubuntu-latest, windows-latest, macos-latest
- **Duration:** ~2-3 minutes per platform
- **Purpose:** Verify build succeeds on all platforms
- **Steps:**
  - Build workspace in debug mode
  - Build workspace in release mode
- **Caching:** Per-OS cache key "build-{os}"
- **Dependencies:** Requires `test` to pass

#### 4. **coverage** (Code coverage)
- **Platform:** ubuntu-latest
- **Duration:** ~3-4 minutes
- **Purpose:** Generate code coverage reports
- **Steps:**
  - Install/cache cargo-tarpaulin (v0.37.4)
  - Generate coverage with 300s timeout
  - Upload to Codecov
- **Caching:**
  - Shared cache key "coverage" for Rust
  - Separate cache for tarpaulin binary
- **Dependencies:** Requires `test` to pass

#### 5. **benchmark** (Performance)
- **Platform:** ubuntu-latest
- **Duration:** Variable (depends on benchmarks)
- **Purpose:** Run performance benchmarks
- **Steps:**
  - Run cargo bench with bencher output format
  - Store results as artifacts (30-day retention)
  - Comment results on PRs
- **Caching:** Shared cache key "benchmark"
- **Dependencies:** Requires `test` to pass

#### 6. **security** (Security audit)
- **Platform:** ubuntu-latest
- **Duration:** ~1-2 minutes
- **Purpose:** Check for security vulnerabilities
- **Steps:**
  - Install/cache cargo-audit (v0.22.2)
  - Run security audit on dependencies
- **Caching:** Separate cache for cargo-audit binary
- **Dependencies:** Requires `lint` to pass (parallel with test)

#### 7. **msrv** (Minimum Rust Version)
- **Platform:** ubuntu-latest
- **Duration:** ~2-3 minutes
- **Purpose:** Verify compatibility with MSRV (1.88)
- **Steps:**
  - Install Rust 1.88 toolchain
  - Check workspace builds with MSRV
- **Caching:** Shared cache key "msrv"
- **Dependencies:** Requires `lint` to pass (parallel with test)

#### 8. **ci-success** (Gate job)
- **Platform:** ubuntu-latest
- **Duration:** ~10 seconds
- **Purpose:** Ensure all CI jobs succeeded
- **Behavior:** Fails if any dependent job fails
- **Dependencies:** All 7 other jobs

### Optimizations

1. **Concurrency Control:**
   - Cancels previous runs on new push to same branch
   - Prevents resource waste on superseded commits

2. **Job Dependencies:**
   - Fast-fail on lint errors (don't run tests if formatting fails)
   - Tests must pass before builds
   - Security/MSRV run in parallel with tests

3. **Caching Strategy:**
   - Uses Swatinem/rust-cache@v2 for intelligent Rust caching
   - Shared cache keys for similar operations
   - Separate caches for tools (tarpaulin, cargo-audit)
   - Cache-on-failure enabled for faster retry cycles

4. **Network Resilience:**
   - `CARGO_NET_RETRY: 10` - Retry failed network operations
   - `RUSTUP_MAX_RETRIES: 10` - Retry rustup operations

### Expected Run Times

| Scenario | Duration | Notes |
|----------|----------|-------|
| Clean run (no cache) | ~12-15 minutes | All jobs from scratch |
| Cached run (no changes) | ~5-7 minutes | Full cache hits |
| Typical PR | ~8-10 minutes | Partial cache hits |
| Lint failure | ~1 minute | Fast-fail on formatting |
| Test failure | ~4-5 minutes | Stops before build |

---

## Release Workflow

**File:** `.github/workflows/release.yml`
**Triggers:**
- Push of tags matching `v*.*.*` (e.g., v0.2.0)
- Release published events
- Manual workflow_dispatch

### Workflow Structure

The release pipeline consists of 3 jobs executed sequentially:

```
create-release → build-release (5 targets) → upload-release
```

### Job Details

#### 1. **create-release** (Optional)
- **Platform:** ubuntu-latest
- **When:** Only on manual `workflow_dispatch` with `create_release: true`
- **Purpose:** Create GitHub release from manual trigger
- **Steps:**
  - Extract version from tag
  - Extract release notes from CHANGELOG.md
  - Create GitHub release (draft or published)
- **Outputs:** `upload_url` for artifact uploads

#### 2. **build-release** (Multi-target)
- **Platforms:** 5 target combinations
- **Duration:** ~10-15 minutes per target
- **Purpose:** Build release binaries for all platforms
- **Targets:**
  - `x86_64-unknown-linux-gnu` (Linux glibc) → tar.gz
  - `x86_64-unknown-linux-musl` (Linux static) → tar.gz
  - `x86_64-pc-windows-msvc` (Windows) → zip
  - `x86_64-apple-darwin` (macOS Intel) → tar.gz
  - `aarch64-apple-darwin` (macOS Apple Silicon) → tar.gz

**Build Matrix:**

| Target | OS | Archive | Static | Notes |
|--------|----|---------|----|-------|
| x86_64-unknown-linux-gnu | ubuntu-latest | tar.gz | No | Most compatible |
| x86_64-unknown-linux-musl | ubuntu-latest | tar.gz | Yes | No libc dependency |
| x86_64-pc-windows-msvc | windows-latest | zip | No | Windows 7+ |
| x86_64-apple-darwin | macos-latest | tar.gz | No | Intel Macs |
| aarch64-apple-darwin | macos-latest | tar.gz | No | M1/M2 Macs |

**Binaries Built:**
- `impulse-server` - Main BBS server
- `impulse-cli` - CLI management tool
- `impconfig` - Configuration utility

**Artifacts Created (per target):**
- `impulse-server-{version}-{target}.{archive}`
- `impulse-cli-{version}-{target}.{archive}`
- `impconfig-{version}-{target}.{archive}`
- `SHA256SUMS-{target}.txt` - Checksums for verification

**Build Process:**
1. Install Rust toolchain with target support
2. Install platform-specific tools (musl-tools for static builds)
3. Build all binaries with `--release --target {target}`
4. Package binaries into archives
5. Generate SHA256 checksums
6. Upload as artifacts (7-day retention)

#### 3. **upload-release**
- **Platform:** ubuntu-latest
- **Duration:** ~2-3 minutes
- **Purpose:** Upload all release artifacts to GitHub
- **Steps:**
  - Download artifacts from all build jobs
  - Combine individual checksum files into one
  - Upload all archives and checksums to release

**Final Release Assets:**
- 15 binary archives (3 binaries × 5 targets)
- 1 combined `SHA256SUMS.txt` file

#### 4. **publish-crates** (Disabled)
- **Platform:** ubuntu-latest
- **Purpose:** Publish crates to crates.io
- **Status:** Disabled by default (`if: false`)
- **Enable When:** Ready for public crate publishing

### Usage Examples

#### Automatic Release (Tag Push)

```bash
# Create and push a tag
git tag v0.2.0
git push origin v0.2.0

# GitHub Actions will automatically:
# 1. Build binaries for all 5 targets
# 2. Create release assets
# 3. Upload to the tag's GitHub Release
```

#### Manual Release (Workflow Dispatch)

```bash
# Trigger via GitHub CLI
gh workflow run release.yml \
  -f tag=v0.2.0 \
  -f create_release=true \
  -f draft=false

# Or via GitHub web UI:
# Actions → Release → Run workflow
# - tag: v0.2.0
# - create_release: ✓
# - draft: ☐
```

#### Create Draft Release

```bash
gh workflow run release.yml \
  -f tag=v0.2.1-rc1 \
  -f create_release=true \
  -f draft=true

# Creates a draft release for review before publishing
```

### Release Notes Extraction

The workflow automatically extracts release notes from `CHANGELOG.md`:

1. **Version-specific:** Looks for `## [x.y.z]` section
2. **Fallback to Unreleased:** Uses `## [Unreleased]` section
3. **Fallback to Generic:** Creates minimal release notes with CHANGELOG link

**Example CHANGELOG.md format:**

```markdown
# Changelog

## [Unreleased]

### Added
- New feature X
- New feature Y

### Fixed
- Bug fix A

## [0.2.0] - 2025-11-26

### Added
- Sprint 16 complete
...
```

### Security Considerations

1. **Token Usage:**
   - Uses `GITHUB_TOKEN` (automatic, read-write for releases)
   - No manual token configuration needed
   - Scoped permissions for release operations

2. **Artifact Integrity:**
   - SHA256 checksums for all binaries
   - Users can verify downloads:
     ```bash
     sha256sum -c SHA256SUMS.txt
     ```

3. **Static Builds:**
   - musl target provides static Linux binaries
   - No system library dependencies
   - Portable across Linux distributions

4. **Permissions:**
   - Workflow requires push access to create releases
   - Manual dispatch requires write access

### Troubleshooting

#### Build Failures

**Problem:** musl build fails
```bash
# Solution: Install musl-tools (automatically done in workflow)
sudo apt-get install musl-tools
```

**Problem:** Cross-compilation fails on macOS
```bash
# Solution: Ensure both targets are installed
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin
```

**Problem:** Windows archive creation fails
```powershell
# Solution: Verify PowerShell version (5.1+)
$PSVersionTable.PSVersion
```

#### Release Creation Failures

**Problem:** "Release already exists"
- **Cause:** Tag pushed twice or manual + tag trigger collision
- **Solution:** Delete existing release or use draft mode

**Problem:** "Cannot extract release notes"
- **Cause:** CHANGELOG.md format doesn't match expected pattern
- **Solution:** Follow Keep a Changelog format or release will use generic notes

**Problem:** Asset upload fails
- **Cause:** Artifact name collision or size limit
- **Solution:** Check artifact names are unique, verify size < 2GB per file

### Monitoring

#### Check Workflow Status

```bash
# List recent workflow runs
gh run list --workflow=release.yml

# Watch current run
gh run watch

# View detailed logs
gh run view <run-id> --log
```

#### Download Release Assets

```bash
# List releases
gh release list

# Download specific release
gh release download v0.2.0

# Download specific asset
gh release download v0.2.0 -p "impulse-server-*.tar.gz"
```

---

## Maintenance

### Updating Workflow Dependencies

The workflows use specific versions of tools and actions:

**GitHub Actions:**
- `actions/checkout@v7` - Checkout code
- `actions/cache@v6` - Generic caching
- `actions/upload-artifact@v7` - Upload artifacts
- `actions/download-artifact@v8` - Download artifacts
- `dtolnay/rust-toolchain@stable` - Install Rust
- `Swatinem/rust-cache@v2` - Rust-specific caching
- `codecov/codecov-action@v7` - Upload coverage
- `softprops/action-gh-release@v3` - Create releases

**Rust Tools:**
- `cargo-tarpaulin@0.37.4` - Coverage generation
- `cargo-audit@0.22.2` - Security auditing

**Update Schedule:**
- Dependabot handles GitHub Actions updates weekly
- Tool versions should be reviewed quarterly
- Major version bumps require testing

### Adding New Binary Crates

To include a new binary in releases:

1. **Update release.yml:**
   ```yaml
   # In build-release job, add to build command
   - name: Build release binaries
     run: cargo build --release --target ${{ matrix.target }} \
       --bin impulse-server \
       --bin impulse-cli \
       --bin impconfig \
       --bin NEW_BINARY_NAME
   ```

2. **Update packaging steps:**
   ```yaml
   # Add to for loop
   for BINARY in impulse-server impulse-cli impconfig NEW_BINARY_NAME; do
   ```

3. **Update documentation:**
   - Update this file
   - Update README.md
   - Update CHANGELOG.md

### Adding New Build Targets

To add a new platform target:

1. **Update build matrix:**
   ```yaml
   matrix:
     include:
       # ... existing targets ...
       - os: ubuntu-latest
         target: aarch64-unknown-linux-gnu
         archive: tar.gz
   ```

2. **Add cross-compilation setup if needed:**
   ```yaml
   - name: Install cross compiler
     if: matrix.target == 'aarch64-unknown-linux-gnu'
     run: |
       sudo apt-get update
       sudo apt-get install -y gcc-aarch64-linux-gnu
   ```

3. **Test locally first:**
   ```bash
   rustup target add aarch64-unknown-linux-gnu
   cargo build --release --target aarch64-unknown-linux-gnu
   ```

---

## Performance Metrics

### CI Workflow

| Metric | Value | Notes |
|--------|-------|-------|
| Average Duration | 8-10 minutes | With cache |
| Cold Start | 12-15 minutes | No cache |
| Fast-fail (lint) | ~1 minute | Format errors |
| Jobs (total) | 8 | Including gate |
| Jobs (parallel) | 5 | Max concurrent |
| Cache Hit Rate | ~80% | Typical PRs |

### Release Workflow

| Metric | Value | Notes |
|--------|-------|-------|
| Average Duration | 50-70 minutes | All 5 targets |
| Per-target Build | 10-15 minutes | Parallel execution |
| Total Artifacts | 16 files | 15 binaries + checksums |
| Total Size | ~100-150 MB | All targets combined |
| Retention | 7 days | Artifact storage |

### Resource Usage

| Resource | CI | Release | Notes |
|----------|----|---------|----|
| Compute Minutes | ~30-40 min | ~50-70 min | GitHub Actions billing |
| Storage | ~500 MB | ~150 MB | Artifacts + cache |
| Network | ~1-2 GB | ~500 MB | Downloads |

---

## Best Practices

### For Developers

1. **Run checks locally before pushing:**
   ```bash
   cargo fmt --all -- --check
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test --workspace --all-features
   ```

2. **Use draft PRs for WIP:**
   - Draft PRs still run CI but signal incomplete work
   - Convert to ready when checks pass

3. **Watch CI logs for failures:**
   ```bash
   gh pr checks  # View PR check status
   gh run watch  # Watch current workflow
   ```

### For Releases

1. **Update CHANGELOG.md before tagging:**
   - Move Unreleased items to new version section
   - Add release date
   - Follow Keep a Changelog format

2. **Test release workflow with draft:**
   ```bash
   gh workflow run release.yml -f tag=v0.2.1-rc1 -f draft=true
   ```

3. **Verify checksums after release:**
   ```bash
   gh release download v0.2.0
   sha256sum -c SHA256SUMS.txt
   ```

4. **Announce releases:**
   - GitHub Discussions
   - Discord/Community channels
   - Social media

### For Maintainers

1. **Monitor workflow performance:**
   - Review GitHub Actions insights
   - Identify slow jobs
   - Optimize cache strategies

2. **Review security audits:**
   - Check cargo-audit results weekly
   - Update vulnerable dependencies promptly
   - File issues for unpatched vulnerabilities

3. **Keep dependencies updated:**
   - Review Dependabot PRs weekly
   - Test grouped updates together
   - Document breaking changes

4. **Plan for growth:**
   - Adjust cache strategies as codebase grows
   - Consider self-hosted runners for faster builds
   - Monitor GitHub Actions quota usage

---

## Support and Troubleshooting

### Common Issues

#### "Cache restore failed"
- **Cause:** Network issues or corrupted cache
- **Solution:** Re-run workflow, cache will rebuild

#### "Test timeout"
- **Cause:** Slow test or infinite loop
- **Solution:** Identify slow test, increase timeout, or fix bug

#### "Clippy warnings as errors"
- **Cause:** New warnings from clippy update
- **Solution:** Fix warnings or add targeted `#[allow(...)]`

#### "MSRV check fails"
- **Cause:** Using features not available in Rust 1.85
- **Solution:** Update MSRV or avoid newer features

### Getting Help

1. **Check workflow logs:**
   ```bash
   gh run view --log-failed
   ```

2. **Search existing issues:**
   - GitHub Actions documentation
   - Rust tooling issues
   - Project issue tracker

3. **Ask for help:**
   - Open issue with workflow logs
   - Tag maintainers for urgent issues
   - Include reproduction steps

---

## Future Enhancements

### Planned Improvements

1. **Matrix optimization:**
   - Dynamic target selection based on changes
   - Skip unchanged platforms

2. **Advanced caching:**
   - Shared cache between workflows
   - Distributed cache for faster restores

3. **Release automation:**
   - Automatic version bumping
   - Changelog generation from commits
   - Discord/Slack notifications

4. **Testing improvements:**
   - Parallel test execution
   - Test sharding for faster runs
   - Flaky test detection

5. **Security enhancements:**
   - SBOM generation
   - Container scanning (when Docker images added)
   - Supply chain security

---

## Conclusion

The Impulse-Next_BBS CI/CD pipelines provide:

- **Fast feedback:** ~1 minute for lint errors, ~5 minutes for test failures
- **Comprehensive checks:** 8 jobs covering testing, security, and compatibility
- **Automated releases:** One-command release creation for 5 platforms
- **High reliability:** Dependency ordering, retry logic, and caching
- **Developer friendly:** Clear logs, PR comments, and artifact downloads

**Next Steps:**
- Review this documentation
- Test workflows with a PR
- Create a test release
- Monitor performance metrics
- Gather team feedback

---

**Document Version:** 1.0.0
**Last Updated:** 2025-11-26
**Maintained By:** Impulse-Next_BBS Team
**Questions:** Open an issue or contact maintainers
