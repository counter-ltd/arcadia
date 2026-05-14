# Push-Arcadia Skill Definition

## Metadata
- **Name:** push-arcadia
- **Path:** `/Users/x/Library/Application Support/Claude/local-agent-mode-sessions/skills-plugin/d268ea45-f349-4103-8ecb-b76bb8016ab1/e58e5220-e1a1-45f4-8b0c-42f64837dd17/skills/push-arcadia/SKILL.md`
- **Compatibility:** Requires git, bash. Must run from Arcadia root directory.

## Description
Commit and push Arcadia main repo and Extensions submodule together in one shot. Analyzes changes, generates descriptive commit messages, handles both repos sequentially, and reports what was pushed. Use this whenever you want to push work to both repos — just say "push arcadia" or "push + commit all".

## Workflow

### Step 1: Check Extensions Submodule Status
- Navigate to Extensions submodule
- Check `git status`
- If changes exist, proceed to Step 2. Otherwise, skip to Step 3.

### Step 2: Commit & Push Extensions
- Stage all changes: `git add -A`
- Auto-generate commit message based on `git diff` analysis
- Commit with message format: `[type]: [description] — [detail]`
- Include footer: `Co-Authored-By: Claude Haiku 4.5 <noreply@anthropic.com>`
- Push to `origin/main`

### Step 3: Check Main Repo Status
- Return to main repo root
- Check `git status`
- If changes exist, proceed to Step 4. Otherwise, skip to Step 5.

### Step 4: Commit & Push Main Repo
- Stage all changes (including submodule ref update): `git add -A`
- Auto-generate commit message based on `git diff` analysis
- Commit with message format: `[type]: [description] — [detail]`
- Include footer: `Co-Authored-By: Claude Haiku 4.5 <noreply@anthropic.com>`
- Push to `origin/development`

### Step 5: Report Results
- Display final commit hash: `git log --oneline -1`
- Report which repo(s) were committed and pushed
- Report commit hashes for both repos

## Commit Message Generation

Messages are auto-generated based on `git diff` analysis:
- Determine commit type: `feat` (new feature) or `fix` (bug fix)
- Write descriptive title with detail line
- Include all Co-Authored-By footer

### Example Messages
- `feat: new extensions (rainbow_indent, rust_syntax, taurine) — add new extension types`
- `fix: AI models UI refinements — improve lifecycle and sidebar integration`
- `feat: documentation updates — AGENTS, CLAUDE, Features catalog`

## Key Rules

1. **Always `git add -A`** before committing (stage all changes)
2. **Extensions submodule commits to `main`**; main repo commits to `development`
3. **Submodule must be committed first**, then main repo reference gets updated
4. **Every commit includes Co-Authored-By footer**
5. **Push immediately after each commit**
6. **Report final commit hash and branch** for verification

## Usage

From Arcadia root directory:
```bash
# User invokes:
push arcadia

# Or equivalently:
push + commit all
```

The skill then:
1. Checks Extensions status
2. Commits Extensions (if changes) → pushes to origin/main
3. Checks main repo status
4. Commits main repo (if changes) → pushes to origin/development
5. Reports commit hashes and branches

## Test Status

**As of 2026-05-13:**
- Skill definition: Valid and accessible
- Both repositories: Clean, no uncommitted changes
- Ready for use when changes exist in either repo
