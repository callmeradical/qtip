# Track Specification: qtip Guard

## Overview
This track involves the implementation of a "guard" service designed to run at the top level of a testable repository (Subject Under Test - SUT). It monitors the SUT's file system for changes, reads a configuration file from the SUT, and automatically generates qtip scenarios into a specified secondary repository.

## Functional Requirements
- **Guard Execution Context**: The service MUST be executed at the root of the SUT.
- **Configuration Parsing**: Load and parse a `.yaml` configuration file from the SUT's root directory.
- **Secondary Repository Mapping**: The configuration MUST specify the location (path or URL) of a secondary repository where generated qtip scenarios will be stored.
- **Real-Time File Watching**: Utilize `chokidar` to monitor specific directories within the SUT for changes.
- **Scenario Generation**: Automatically generate qtip-compliant scenario definitions (YAML) based on SUT changes and acceptance criteria defined in the configuration.
- **Scenario Storage**: Write the generated scenarios directly to the specified secondary repository.

## Non-Functional Requirements
- **Performance**: Efficient file watching and minimal latency in scenario generation.
- **Reliability**: Graceful handling of file system errors and configuration mismatches.
- **Maintainability**: Adherence to project conventions (TypeScript, Zod validation).

## Acceptance Criteria
- [ ] The "guard" service correctly initializes and starts watching the SUT's root directory.
- [ ] The service correctly identifies the secondary scenario repository from the SUT's configuration file.
- [ ] Modifying a watched source file in the SUT triggers the automatic generation of a corresponding qtip scenario in the secondary repository.
- [ ] The generated scenarios are valid and correctly formatted for the qtip platform.
- [ ] Invalid configurations or file system errors (e.g., missing secondary repo) are handled gracefully with clear error messages.

## Out of Scope
- Direct Git synchronization for the secondary repository (initial support assumes a local or pre-cloned path).
- Support for multiple SUTs simultaneously in one guard process.
