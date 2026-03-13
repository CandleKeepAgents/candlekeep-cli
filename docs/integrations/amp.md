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

### Real-World Example: Cross-Standard Compliance Analysis

In this example, a user uploaded two standards PDFs (MISRA C:2023 and SEI CERT C 2016) and asked Amp to analyze how they overlap. Here's how CandleKeep + Amp handled it:

**Step 1 — Upload standards:**
```bash
ck items add MISRA-C-2023.pdf
ck items add SEI_CERT_C_Coding_Standard_2016_Edition.pdf
```

**Step 2 — Ask a cross-standard question:**
```
You: "What is the overlap between MISRA C and CERT C?"
```

**Step 3 — Amp automatically:**
1. Ran `ck items list --json` → found both documents
2. Ran `ck items toc <id> --json` on each → located relevant sections
3. Read Appendix H of MISRA (undefined behavior mappings) and Appendix C of CERT C (undefined behavior → rule mappings)
4. Read specific rule pages from both standards in parallel (e.g., MISRA Rule 11.3 on page 123 vs. CERT EXP36-C on page 93)
5. Produced a cross-reference traceability matrix:

| Topic | MISRA C:2023 | CERT C | CWE |
|---|---|---|---|
| Pointer casting / alignment | Rule 11.3, 11.4 | EXP36-C, INT36-C | CWE-704 |
| Null pointer dereference | Rule 1.3 (UB) | EXP34-C | CWE-476 |
| Uninitialized memory read | Rule 9.1 | EXP33-C | CWE-457 |
| Signed integer overflow | Rule 12.4 | INT32-C | CWE-190 |
| Use-after-free | Rule 18.6, 22.6 | MEM30-C | CWE-416 |
| Buffer overflow (string ops) | Rule 21.17, 21.18 | STR31-C | CWE-120 |
| Data races | Dir 5.1 | CON32-C, CON33-C | CWE-362 |
| Command injection via system() | Rule 21.21 | ENV33-C | CWE-78 |

**Step 4 — Follow-up question with nuance:**
```
You: "If MISRA Rule 11.3 passes, can I skip scanning for CERT EXP36-C?"
```

Amp read the specific rule pages from both documents and identified that while MISRA 11.3 is **broader** (bans all pointer type conversions), CERT EXP36-C catches additional `void*` round-trip alignment issues not in MISRA's scope — so the answer was "mostly yes, but CERT adds edge cases."

**Step 5 — Generate a comprehensive analysis document:**
```
You: "What issues that are undefined or unspecified in ISO C are solved in MISRA C?"
```

Amp read MISRA Appendix G (~100+ implementation-defined behaviors), Appendix H (~203 undefined behaviors, ~58 unspecified behaviors), and produced a complete analysis showing how MISRA C makes ISO C deterministic across three layers:
- **Layer 1:** Undefined behavior → banned outright via specific rules
- **Layer 2:** Unspecified behavior → constrained to be order-independent
- **Layer 3:** Implementation-defined behavior → must document per Dir 1.1

The resulting document was saved back to CandleKeep for future reference across sessions.

This entire workflow — from uploading two PDFs to producing a publication-ready cross-standard analysis — was done conversationally in a single Amp session, with every finding cited to specific document pages.

## Compatibility

- The `ck` CLI runs on macOS, Linux, and Windows
- Amp skill uses only `ck` CLI commands via shell — no additional dependencies
- Works with Amp CLI and Amp in VS Code / Cursor

## Links

- [CandleKeep Cloud](https://getcandlekeep.com)
- [CandleKeep CLI](https://github.com/CandleKeepAgents/candlekeep-cli)
- [Amp](https://ampcode.com)
- [CandleKeep Marketplace (Claude Code)](https://github.com/CandleKeepAgents/candlekeep-marketplace)
