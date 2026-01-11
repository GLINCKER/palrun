# Tutorial: Your First Command Palette

**Time**: 10 minutes | **Level**: Beginner

Learn how to install Palrun and execute your first command using the interactive command palette.

## What You'll Learn

- How to install Palrun on your system
- How to launch the command palette
- How to search for commands
- How to execute a command

## Prerequisites

- A terminal emulator (Terminal, iTerm2, Alacritty, Windows Terminal, etc.)
- Either Rust (for cargo install) or Node.js (for npm install)
- A project with a `package.json`, `Cargo.toml`, or `Makefile`

## Step 1: Install Palrun

Choose your preferred installation method:

### Option A: Using Cargo (Recommended)

If you have Rust installed:

```bash
cargo install palrun
```

Wait for the installation to complete. This may take a few minutes as it compiles from source.

### Option B: Using NPM

If you prefer npm:

```bash
npm install -g @glinr/palrun
```

This downloads a pre-built binary for your platform.

## Step 2: Verify Installation

Check that Palrun is installed correctly:

```bash
palrun --version
```

You should see output like:

```
palrun 0.1.0
```

If you see "command not found", check that your PATH is configured correctly. See the [Installation Guide](../installation.md#binary-not-found-after-installation) for help.

## Step 3: Navigate to a Project

Open your terminal and navigate to any project directory. For this tutorial, we'll use a Node.js project as an example:

```bash
cd ~/projects/my-app
```

Make sure the directory contains a `package.json` file:

```bash
ls package.json
```

## Step 4: Launch the Command Palette

Now, launch Palrun:

```bash
palrun
```

You should see an interactive interface appear:

```
+-----------------------------------------------------------------------------+
| Search: _                                                                   |
+-----------------------------------------------------------------------------+
| > npm run dev          [npm]  Start development server                     |
|   npm run build        [npm]  Build for production                         |
|   npm run test         [npm]  Run test suite                               |
|   npm run lint         [npm]  Lint code                                    |
+-----------------------------------------------------------------------------+
| 4 commands found | Use arrows to navigate, Enter to execute, Esc to quit   |
+-----------------------------------------------------------------------------+
```

**Congratulations!** You've launched your first command palette.

## Step 5: Navigate the Command List

Try these navigation keys:

1. Press **Down Arrow** to move to the next command
2. Press **Up Arrow** to move back
3. Notice how the `>` indicator moves with your selection

The selected command is highlighted and ready to execute.

## Step 6: Search for a Command

Now let's try the fuzzy search:

1. Type `dev` in the search box
2. Watch as the list filters to show only matching commands
3. Notice that "npm run dev" appears at the top

The search is fuzzy, so you don't need to type the exact name. Try typing just `d` or `dv` - it still finds "dev"!

## Step 7: Clear the Search

Press **Ctrl+U** to clear the search input. The full command list reappears.

## Step 8: Execute a Command

Let's execute the "npm run dev" command:

1. Type `dev` to filter the list
2. Make sure "npm run dev" is selected (it should be at the top)
3. Press **Enter**

Palrun will exit and execute the command in your terminal. You should see your development server starting!

```
> my-app@1.0.0 dev
> vite

  VITE v5.0.0  ready in 500 ms

  ➜  Local:   http://localhost:5173/
  ➜  Network: use --host to expose
```

**Success!** You've executed your first command using Palrun.

## Step 9: Stop the Server and Try Again

Press **Ctrl+C** to stop the development server.

Now launch Palrun again:

```bash
palrun
```

This time, try executing a different command:

1. Type `test`
2. Select "npm run test"
3. Press **Enter**

Watch as your test suite runs!

## Step 10: Quit Without Executing

Launch Palrun one more time:

```bash
palrun
```

This time, we'll quit without executing anything:

1. Press **Escape** (or **Ctrl+C**)
2. Palrun exits and returns you to your shell prompt

## What You've Learned

In this tutorial, you've learned:

- ✓ How to install Palrun using cargo or npm
- ✓ How to launch the interactive command palette
- ✓ How to navigate the command list with arrow keys
- ✓ How to search for commands using fuzzy matching
- ✓ How to execute commands by pressing Enter
- ✓ How to quit without executing

## Troubleshooting

### No Commands Found

If Palrun shows "No commands found":

1. Make sure you're in a project directory
2. Check that `package.json` (or another supported config file) exists
3. Try running `palrun scan` to see what Palrun detects

### Command Not Found

If you see "palrun: command not found":

1. Check your PATH includes `~/.cargo/bin` (for cargo install)
2. Try opening a new terminal window
3. See the [Installation Guide](../installation.md) for platform-specific help

### TUI Looks Broken

If the interface looks garbled:

1. Make sure your terminal supports colors
2. Try a modern terminal emulator (iTerm2, Alacritty, Windows Terminal)
3. Check that your terminal is at least 80 characters wide

## Next Steps

Now that you know the basics, continue with:

- [Tutorial 2: Setting Up Your Workflow](02-setting-up-workflow.md) - Configure shell integration and customize Palrun
- [Tutorial 3: Working with a Node.js Project](03-nodejs-project.md) - Learn more about npm script discovery
- [How-to Guide: Customize the Theme](../guides/customize-theme.md) - Change colors and appearance

## Practice Exercise

Before moving on, practice what you've learned:

1. Navigate to a different project
2. Launch Palrun
3. Try searching for different commands
4. Execute at least 3 different commands
5. Practice quitting with Escape

The more you practice, the faster you'll become!

