# Codex API Key Manager Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a desktop tool that manages multiple Codex API key profiles and switches the active Codex `auth.json` and `config.toml` files together.

**Architecture:** Keep all file system access and config mutation inside Tauri commands written in Rust, and keep the Vue layer focused on profile management and user actions. Persist the profile list in the app data directory so switching does not depend on editing the app source.

**Tech Stack:** Tauri 2, Rust, Vue 3, TypeScript

---

## Chunk 1: Rust Core

### Task 1: Add failing Rust tests for profile storage and config switching

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write the failing tests**

Add tests for:
- saving and loading profile data from an app-owned JSON file
- resolving Codex config paths for Windows and macOS
- updating `OPENAI_API_KEY` in `auth.json`
- updating `base_url` in `config.toml`

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test`
Expected: FAIL because profile storage and config switching code does not exist yet.

- [ ] **Step 3: Write minimal implementation**

Implement Rust helpers and Tauri commands for:
- profile persistence
- platform-aware Codex file path resolution
- reading current active values
- switching the active profile by updating both config files

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test`
Expected: PASS

### Task 2: Expose clean command payloads for the frontend

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add command-level tests where practical**
- [ ] **Step 2: Implement serializable request and response models**
- [ ] **Step 3: Return human-readable error messages for missing files, parse failures, and write failures**
- [ ] **Step 4: Run `cargo test` again**

## Chunk 2: Vue UI

### Task 3: Add failing frontend tests for profile workflow

**Files:**
- Create: `src/components/ProfileManager.test.ts`
- Create: `src/components/ProfileManager.vue`
- Modify: `package.json`

- [ ] **Step 1: Add a frontend test runner and write failing tests**

Cover:
- rendering saved profiles
- validating required fields before save
- disabling switch while an action is running

- [ ] **Step 2: Run test to verify it fails**

Run: `pnpm test`
Expected: FAIL because the component and wiring do not exist yet.

- [ ] **Step 3: Build the profile manager UI**

Create a focused component that supports:
- listing profiles
- adding and editing name, key, and base URL
- deleting profiles
- switching the active profile
- showing status and error text

- [ ] **Step 4: Run `pnpm test` to verify it passes**

### Task 4: Replace the template app shell

**Files:**
- Modify: `src/App.vue`
- Modify: `src/main.ts`

- [ ] **Step 1: Mount the new manager screen**
- [ ] **Step 2: Keep styling responsive and simple**
- [ ] **Step 3: Run `pnpm test` and `pnpm build`**

## Chunk 3: Verification

### Task 5: Run full verification and inspect behavior

**Files:**
- Modify as needed based on failures

- [ ] **Step 1: Run `cargo test`**
- [ ] **Step 2: Run `pnpm test`**
- [ ] **Step 3: Run `pnpm build`**
- [ ] **Step 4: Run `pnpm tauri build` or `pnpm tauri dev` for end-to-end validation if the environment allows it**
- [ ] **Step 5: Verify the app updates representative `auth.json` and `config.toml` files correctly**
