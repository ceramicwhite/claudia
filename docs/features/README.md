# Features Documentation

This section documents all the features and capabilities of Claudia, the desktop GUI for Claude Code CLI.

## Contents

### Core Features
- [Claude Code Integration](./claude-integration.md) - Running and managing Claude sessions
- [Agent System](./agents.md) - Custom AI agents with sandboxing
- [Widget System](./widgets.md) - Rendering AI tool outputs with specialized components
- [MCP Servers](./mcp-servers.md) - Model Context Protocol integration

### Session Management
- [Session Control](./session-control.md) - Starting, stopping, and managing sessions
- [Timeline Navigation](./timeline.md) - Browse session history
- [Checkpoints](./checkpoints.md) - Save and restore session states
- [Output Streaming](./output-streaming.md) - Real-time session output

### Development Tools
- [File Editor](./file-editor.md) - Built-in code editor
- [Terminal Integration](./terminal.md) - Command execution
- [Git Integration](./git.md) - Version control features
- [Project Management](./project-management.md) - Organize and switch projects

### Analytics & Monitoring
- [Usage Dashboard](./usage-dashboard.md) - Token usage and analytics
- [Performance Metrics](./performance.md) - Session performance tracking
- [Error Tracking](./error-tracking.md) - Error monitoring and debugging

### Security
- [Sandboxing](./sandboxing.md) - Agent security profiles
- [Permissions](./permissions.md) - File system and network access control
- [Violation Tracking](./violations.md) - Security violation monitoring

### User Interface
- [Themes](./themes.md) - Dark and light mode support
- [Keyboard Shortcuts](./shortcuts.md) - Productivity shortcuts
- [Customization](./customization.md) - UI preferences and settings

## Feature Highlights

### 🤖 Claude Code Integration
Seamlessly run Claude Code sessions with full control over model selection, parameters, and execution environment.

### 🛡️ Security-First Design
Every agent runs in a sandboxed environment with configurable security profiles, protecting your system from unintended actions.

### 📊 Real-Time Analytics
Track token usage, session performance, and costs with detailed analytics and visualizations.

### 🔄 Timeline Navigation
Navigate through your session history with checkpoints, allowing you to explore different solution paths.

### 🎨 Widget System
Beautiful, purpose-built widgets for displaying AI tool outputs - from file operations to web searches.

### 🔌 MCP Integration
Connect to Model Context Protocol servers for extended capabilities and tool integrations.

## Quick Start

1. **Start a Session**: Click "New Session" and select your project
2. **Choose a Model**: Select Claude model and parameters
3. **Run Commands**: Type your prompt and watch Claude work
4. **Review Output**: Widgets display results beautifully
5. **Save Progress**: Create checkpoints to save your work

## Architecture Overview

Claudia is built with:
- **Frontend**: React + TypeScript + Tailwind CSS
- **Backend**: Rust + Tauri for native performance
- **Database**: SQLite for local data storage
- **Security**: Platform-specific sandboxing

See [Architecture Documentation](../architecture/) for detailed technical information.