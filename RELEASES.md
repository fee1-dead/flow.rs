# Version 0.2

### Added

### Changed
 - Fixed wrong implementation of envelope RLP encoding and signing
 - Changed various request definitions to use generics
 - Changed various `Box<dyn Error + Send + Sync>` uses to `crate::error::BoxError`
 - Moved error types to `crate::error`
 - Added more examples and tests

### Removed

# Version 0.1

Initial release