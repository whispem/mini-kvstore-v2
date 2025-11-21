name: Bug report
about: Create a report to help us improve
title: '[BUG] '
labels: bug
assignees: ''
---

**Describe the bug**
A clear and concise description of what the bug is.

**To Reproduce**
Steps to reproduce the behavior:
1. Run command '...'
2. Send request '...'
3. Observe error '...'

**Expected behavior**
A clear and concise description of what you expected to happen.

**Actual behavior**
What actually happened.

**Logs/Error messages**
Paste relevant logs or error messages here

**Environment:**
 - OS: [e.g. macOS 14.6.1, Ubuntu 22.04]
 - Rust version: [e.g. 1.75.0]
 - mini-kvstore-v2 version: [e.g. 0.3.0]
 - Architecture: [e.g. x86_64, aarch64]

**Configuration:**
# Paste your configuration (environment variables, command-line flags)
PORT=8000 cargo run --release

**Data directory state:**
# Output of: ls -lh data/
total 64M
-rw-r--r-- 1 user user 32M Nov 21 10:00 segment-0000.dat
-rw-r--r-- 1 user user 32M Nov 21 10:05 segment-0001.dat

**Additional context**
Add any other context about the problem here.

**Possible solution**
If you have an idea of what might fix the issue, please share it here.
