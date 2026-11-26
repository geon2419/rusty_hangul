test:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo test
    bun test

bench:
    #!/usr/bin/env bash
    set -euo pipefail
    bun run bench:report
