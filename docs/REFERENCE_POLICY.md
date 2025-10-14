# Reference Documentation Policy

## Important Notice

Reference documentation from `/data_drive/docs/references/` should **NOT** be committed to this repository unless explicitly copied in by the user. These documents may contain company confidential information.

## Guidelines

1. **External References**: Documentation stored in `/data_drive/docs/references/` is maintained outside the project repository
2. **Confidentiality**: Reference materials may contain proprietary or confidential information
3. **Explicit Permission**: Only commit reference documents when the user explicitly copies them into the project
4. **Git Ignore**: The `.gitignore` file should exclude any reference document patterns to prevent accidental commits

## Current Setup

- XM125 reference documentation is located at: `/data_drive/docs/references/`
- Project documentation links to the external location
- No reference documents are stored within the project repository

## If Reference Documents Are Needed in Repository

If reference documents need to be included in the repository:

1. User must explicitly copy them into a project directory
2. Ensure documents are not confidential or have appropriate permissions
3. Update `.gitignore` if needed to allow specific documents
4. Document the inclusion in commit messages

This policy helps maintain security and prevents accidental disclosure of confidential information.
