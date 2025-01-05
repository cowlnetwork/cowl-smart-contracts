# Cowl Smart Contracts - Setup and Installation

## Prerequisites

- Git installed on your system.
- SSH access to GitHub or your GitHub SSH key added to your environment.

## Steps to Clone the Repository and Initialize Submodules

### 1. **Clone the Repository**

First, clone the main repository:

```bash
git clone git@github.com:cowlnetwork/cowl-smart-contracts.git
cd cowl-smart-contracts
```

### 2. **Initialize and Update Submodules**

Once inside the repository directory, you need to initialize and update the submodules. The repository uses several submodules, which include:

- `cowl-vesting`
- `cowl-swap`
- `cowl-cli`

To initialize and fetch these submodules, run the following commands:

```bash
git submodule update --init --recursive
```

This will initialize and fetch the submodules recursively, ensuring all submodules and their specific branches (e.g., `dev`) are checked out correctly.

### 3. **Verify Submodule Status**

After updating the submodules, you can check the status of the submodules with:

```bash
git submodule status
```

This will show you the current commit of each submodule.

### 4. **Working with Submodules**

To pull the latest updates for the submodules in the future, use:

```bash
git submodule update --remote --merge
```

This will ensure that each submodule is updated to the latest commit on its tracked branch (e.g., `dev`).

### 5. **Additional Setup for Development (Optional)**

If you are contributing to development or need to make changes, follow these additional steps:

#### Add Submodule Branches (if needed)

If you need to manually add or update submodules, use these commands to add specific branches:

```bash
git submodule add -b dev git@github.com:cowlnetwork/cowl-vesting.git cowl-vesting
git submodule add -b dev git@github.com:cowlnetwork/cowl-swap.git cowl-swap
git submodule add -b dev git@github.com:cowlnetwork/cowl-cli.git cowl-cli
```

#### Pulling Latest Changes for Submodules

If you need to pull updates for all submodules:

```bash
git submodule update --remote --recursive
```

#### Removing Submodules (if necessary)

If you need to remove a submodule, use:

```bash
git submodule deinit <submodule-name>
git rm <submodule-name>
git commit -m "Removed submodule <submodule-name>"
```

---

## Troubleshooting

### 1. **Git SSH Permissions**

Ensure you have SSH access to the required GitHub repositories. If you encounter issues with permission denied errors, you might need to configure your SSH keys for GitHub.

### 2. **Missing Submodules After Cloning**

If some submodules are not properly initialized after cloning, you can re-run the initialization and update command:

```bash
git submodule update --init --recursive
```

---
