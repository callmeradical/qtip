# qtip Guard

`qtip-guard` is a service that monitors your project's file system and automatically generates qtip scenarios based on source code changes and predefined acceptance criteria.

## How it Works

1. **Watch**: The guard service monitors the paths specified in your configuration.
2. **Match**: When a file changes, the guard checks if there's a corresponding mapping in the configuration.
3. **Discover**: If a new file is added, the guard automatically generates a default scenario for it.
4. **Generate**: If a match is found (or a new file is discovered), a qtip scenario (YAML) is generated.
5. **Store**: The generated scenario is saved into a specified secondary repository.

## Automatic Scenario Discovery

`qtip-guard` automatically detects when new files are added to your project. For every new file that matches your `watched_paths`, a corresponding scenario is generated in the `secondary_repo` with a `[DISCOVERY TEMPLATE]` label. This allows you to start validating new features immediately.

## Automated Evaluation (Git Hooks)

You can integrate qtip evaluation directly into your Git workflow to ensure high-integrity changes.

### Installation

To install a Git hook, use the following command:

```bash
# Install a pre-commit hook
npm run qtip:hook install pre-commit

# Install a pre-push hook
npm run qtip:hook install pre-push
```

### How it Works

Once installed, the hook will run `npm run qtip:evaluate` before every commit (or push). If the evaluation fails, the Git action will be blocked, preventing broken code from entering the repository.

## Configuration

Create a `qtip-guard.yaml` file in the root of your repository:

```yaml
# Path to the repository where generated scenarios will be stored
secondary_repo: "./my-scenarios-repo"

# Directories to monitor for changes (glob patterns supported by chokidar)
watched_paths:
  - "src/**/*.ts"

# Mapping from source files to scenario definitions
scenarios_mapping:
  - path: "src/auth/login.ts"
    scenario_name: "User Login Flow"
    acceptance_criteria:
      - "User can successfully log in with valid credentials"
      - "User receives an error with invalid credentials"
      - "Brute-force protection triggers after 5 failed attempts"
```

## Usage

Run the guard service using npm:

```bash
npm run qtip:guard
```

By default, it looks for `qtip-guard.yaml` in the current directory. You can specify a custom configuration path as an argument:

```bash
npm run qtip:guard custom-config.yaml
```

## Installation

Ensure you have the dependencies installed in your project:

```bash
npm install qtip
```

(Note: `qtip-guard` is currently part of the core `qtip` package).
