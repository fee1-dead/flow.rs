# Release 1.0.0

### Changed
 - Added documentation about compiler bugs which caused issues for some functions
 - `TonicHyperFlowClient` connection methods now connects to the endpoint instead of initializing lazily.
    - Previous functions are renamed to `*_lazy`. 

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