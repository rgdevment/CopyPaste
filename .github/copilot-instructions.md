# GitHub Copilot Custom Instructions

## Commit Messages
- **Format**: Always use the Conventional Commits standard: `<type>(<scope>): <description>`.
- **Language**: Always generate messages in English.
- **Types allowed**:
  - `feat`: A new feature for the user.
  - `fix`: A bug fix.
  - `docs`: Documentation only changes.
  - `style`: Changes that do not affect the meaning of the code (white-space, formatting, etc.).
  - `refactor`: A code change that neither fixes a bug nor adds a feature.
  - `perf`: A code change that improves performance.
  - `chore`: Updating build tasks, package manager configs, etc.
- **Rules**:
  - Use the imperative mood ("Add", not "Added").
  - No period (.) at the end of the description.
  - Keep the description under 50 characters.
