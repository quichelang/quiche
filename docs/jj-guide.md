# JJ (Jujutsu) Quick Reference

## Essential Commands

```bash
# View history
jj log -n 10              # Recent 10 commits
jj log --no-graph         # Without ASCII graph

# Status and changes
jj st                     # Status (what's changed)
jj diff                   # Show uncommitted changes

# Commit workflow (jj auto-tracks changes)
jj describe -m "message"  # Set message on current working copy
jj new                    # Create new empty commit, move to it

# Bookmarks (like git branches)
jj bookmark set main      # Set main to current commit
jj bookmark set main -r @-  # Set main to parent commit

# Push
jj git push --bookmark main

# Amending previous commits
jj describe <change-id> -m "new message"  # Change any commit's message
jj squash                 # Squash working copy into parent
```

## Key Differences from Git

- **No staging area**: All changes are automatically tracked
- **Working copy is a commit**: `@` is always a commit you're editing
- **Immutable history**: Old commits stay until explicitly abandoned
- **Change IDs**: Use short prefixes like `pozmzxtl` instead of commit hashes

## Common Workflows

### Making changes
```bash
# Edit files, then:
jj describe -m "feat: my change"
jj new  # Start fresh working copy
jj bookmark set main -r @-  # Point main to the completed commit
jj git push --bookmark main
```

### Fix a commit that's not HEAD
```bash
jj describe <change-id> -m "fixed message"
# Changes automatically rebase descendants
```

### View a specific commit
```bash
jj show <change-id> --stat
jj diff -r <change-id>
```
