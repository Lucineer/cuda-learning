# cuda-learning

Experience-based learning — credit assignment, lesson extraction, curriculum progression, generalization (Rust)

Part of the Cocapn cognitive layer — how agents think, decide, and learn.

## What It Does

### Key Types

- `Experience` — core data structure
- `Lesson` — core data structure
- `CreditAssignment` — core data structure
- `Curriculum` — core data structure
- `CurriculumStage` — core data structure
- `LearningEngine` — core data structure
- _and 1 more (see source)_

## Quick Start

```bash
# Clone
git clone https://github.com/Lucineer/cuda-learning.git
cd cuda-learning

# Build
cargo build

# Run tests
cargo test
```

## Usage

```rust
use cuda_learning::*;

// See src/lib.rs for full API
// 13 unit tests included
```

### Available Implementations

- `Experience` — see source for methods
- `Lesson` — see source for methods
- `CreditAssignment` — see source for methods
- `Curriculum` — see source for methods
- `LearningEngine` — see source for methods

## Testing

```bash
cargo test
```

13 unit tests covering core functionality.

## Architecture

This crate is part of the **Cocapn Fleet** — a git-native multi-agent ecosystem.

- **Category**: cognition
- **Language**: Rust
- **Dependencies**: See `Cargo.toml`
- **Status**: Active development

## Related Crates

- [cuda-confidence-cascade](https://github.com/Lucineer/cuda-confidence-cascade)
- [cuda-deliberation](https://github.com/Lucineer/cuda-deliberation)
- [cuda-reflex](https://github.com/Lucineer/cuda-reflex)
- [cuda-goal](https://github.com/Lucineer/cuda-goal)
- [cuda-fusion](https://github.com/Lucineer/cuda-fusion)
- [cuda-attention](https://github.com/Lucineer/cuda-attention)
- [cuda-emotion](https://github.com/Lucineer/cuda-emotion)
- [cuda-narrative](https://github.com/Lucineer/cuda-narrative)
- [cuda-skill](https://github.com/Lucineer/cuda-skill)

## Fleet Position

```
Casey (Captain)
├── JetsonClaw1 (Lucineer realm — hardware, low-level systems, fleet infrastructure)
├── Oracle1 (SuperInstance — lighthouse, architecture, consensus)
└── Babel (SuperInstance — multilingual scout)
```

## Contributing

This is a fleet vessel component. Fork it, improve it, push a bottle to `message-in-a-bottle/for-jetsonclaw1/`.

## License

MIT

---

*Built by JetsonClaw1 — part of the Cocapn fleet*
*See [cocapn-fleet-readme](https://github.com/Lucineer/cocapn-fleet-readme) for the full fleet roadmap*
