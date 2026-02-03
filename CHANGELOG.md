# Changelog

## [0.3.0] - 2026-02-03

### Added
- **Book Enrichment System**: New commands to manage document metadata
  - `ck items enrich <id>` - Update title, author, description with confidence score
  - `ck items flag <id>` - Flag items needing metadata enrichment
- **Enhanced List Display**:
  - New "Enrich" column showing enrichment status (⚠/✓/-)
  - "Enrichment Queue" section at bottom showing items needing review
  - Items sorted by page count (smallest first in queue)
- **JSON Output Enhancements**:
  - `enrichmentQueue` array in `--json` output
  - New fields: `author`, `needsEnrichment`, `enrichmentConfidence`, `enrichedAt`

### Changed
- Items list now shows 5 columns: ID, Title, Pages, Status, Enrich

## [0.2.1] - 2026-01-30

### Changed
- Added release safeguards and documentation

## [0.2.0] - 2026-01-30

### Added
- Initial CLI release with core functionality
- `ck auth login/logout/whoami` commands
- `ck items list/add/read/toc/remove` commands
- Markdown file upload support
