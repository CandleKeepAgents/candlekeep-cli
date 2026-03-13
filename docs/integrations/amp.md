# Using CandleKeep with Amp

## Overview

CandleKeep CLI works seamlessly with [Amp](https://ampcode.com), an AI coding agent. This guide shows how to set up CandleKeep as an Amp skill so your AI agent can search, read, and cross-reference your document library during coding sessions.

## Why Use CandleKeep with Amp?

- **Cross-standard compliance**: Upload MISRA, CERT-C, AUTOSAR, CWE, and other standards PDFs — Amp can cross-reference rules across all of them
- **Persistent document library**: Documents are stored in CandleKeep Cloud, accessible across all Amp sessions
- **Cited research**: Every answer includes document title and page number citations
- **Parallel research**: Amp can read multiple documents simultaneously for complex queries

## Prerequisites

1. A CandleKeep Cloud account ([getcandlekeep.com](https://getcandlekeep.com))
2. The `ck` CLI installed:
   ```bash
   # macOS (Homebrew)
   brew tap CandleKeepAgents/candlekeep
   brew install candlekeep-cli

   # From source
   cargo install candlekeep-cli

   # Or download binaries from GitHub Releases
   ```
3. [Amp](https://ampcode.com) installed
4. Authenticate:
   ```bash
   ck auth login
   ```

## Installation

### Option 1: Global skill (available in all projects)

Copy the skill folder to your global Amp skills directory:

```
~/.agents/skills/candlekeep/
├── SKILL.md
└── README.md
```

### Option 2: Project-level skill (per-project)

Add the skill folder to your project:

```
your-project/
├── skills/
│   └── candlekeep/
│       ├── SKILL.md
│       └── README.md
```

## Quick Start

1. Upload documents to your library:
   ```bash
   ck items add ~/standards/MISRA-C-2023.pdf
   ck items add ~/standards/CERT-C.pdf
   ck items add ~/standards/AUTOSAR-CPP14.pdf
   ```

2. Verify your library:
   ```bash
   ck items list
   ```

3. In Amp, just ask naturally:
   - *"What does my library say about pointer casting?"*
   - *"Cross-reference MISRA Rule 11.3 across all my standards"*
   - *"Which of my standards cover null pointer dereferencing?"*

## How It Works

1. When you mention your documents/library, Amp loads the CandleKeep skill
2. The skill runs `ck items list --json` to see what's in your library
3. It identifies relevant documents by title and description
4. It reads specific pages using `ck items read <id>:<pages> --json`
5. It synthesizes findings with citations back to you

For complex queries spanning multiple documents, Amp dispatches parallel readers — each focused on specific documents and sub-questions — then merges the results.

## Example Workflows

### Standards Compliance Research

```
You: "I have a Parasoft violation for MISRA Rule 11.3 — what do my other standards say about the same issue?"

Amp: [searches library → finds MISRA, CERT-C, AUTOSAR PDFs → reads relevant sections → produces cross-reference table]
```

### Uploading and Researching

```
You: "Upload C:\docs\IEC-62443.pdf to my library"
Amp: ck items add "C:\docs\IEC-62443.pdf"

You: "Now cross-reference it with my MISRA document on memory safety rules"
Amp: [reads both documents in parallel → produces comparison]
```

## Compatibility

- The `ck` CLI runs on macOS, Linux, and Windows
- Amp skill uses only `ck` CLI commands via shell — no additional dependencies
- Works with Amp CLI and Amp in VS Code / Cursor

## Links

- [CandleKeep Cloud](https://getcandlekeep.com)
- [CandleKeep CLI](https://github.com/CandleKeepAgents/candlekeep-cli)
- [Amp](https://ampcode.com)
- [CandleKeep Marketplace (Claude Code)](https://github.com/CandleKeepAgents/candlekeep-marketplace)
