# Changelog

## [0.4.0] - 2026-02-04

### Added
- **Markdown Document Support**: Full CRUD operations for markdown documents
  - `ck items create <title>` - Create new markdown documents with `--content` flag
  - `ck items get <id>` - Retrieve markdown content for a document
  - `ck items put <id>` - Update markdown content for existing documents
- Table of contents extraction now works for both PDF and markdown documents
- Real-time content versioning with page tracking

### Changed
- `ck items list` now shows markdown documents alongside PDFs
- `ck items toc` supports both PDF and markdown documents
- `ck items read` works with markdown page ranges

## [0.3.1] - 2026-02-03

### Added
- **TOC Support in Enrich**: New `--toc` option for `ck items enrich` command
  - Accepts JSON array: `--toc '[{"title":"Chapter 1","page":1,"level":1}]'`
  - Validates TOC entries (non-empty titles, page >= 1, level >= 1)
  - Saves TOC to item metadata for navigation

### Changed
- `ck items enrich` now requires at least one of: --title, --author, --description, or --toc

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
