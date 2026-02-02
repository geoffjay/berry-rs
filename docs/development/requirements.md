# Berry-rs Requirements

This document outlines the build requirements for the Berry-rs project.

- types must never be recreated in the individual crates, they must be loaded from a common `berry` crate
- individual type implementations should be given their own file and re-exported by the appropriate module
- the `berry` crate should be the only crate that re-exports the types
- in general modules should be leveraged to provide a clear separation of concerns
- small, focused, multi-file implementations should be preferred over large, complex single-file implementations
- testing is a critical part of the development process and should be done alongside implementation and used to verify
