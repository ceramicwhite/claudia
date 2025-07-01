# Claudia CC Agents

This document provides comprehensive information about Claudia's CC Agents feature, including available agents, technical implementation, and usage guides.

## Table of Contents

1. [Overview](#overview)
2. [Available Agents](#available-agents)
3. [Using Agents](#using-agents)
4. [Agent File Format](#agent-file-format)
5. [Technical Implementation](#technical-implementation)
6. [Creating Custom Agents](#creating-custom-agents)
7. [Contributing Agents](#contributing-agents)

## Overview

CC Agents are pre-built AI agents for Claudia powered by Claude Code. They provide specialized functionality for common development tasks like Git automation, security scanning, and test generation.

### Key Features

- **Pre-built Agents**: Ready-to-use agents for common tasks
- **Custom Agents**: Create your own specialized agents
- **Import/Export**: Share agents via GitHub or local files
- **Permission Control**: Granular permissions for file, network, and sandbox access
- **Model Selection**: Choose between Opus, Sonnet, and Haiku models
- **Version Control**: Agent configurations include version metadata

## Available Agents

### 🎯 Git Commit Bot
- **Icon**: 🤖 `bot`
- **Model**: Sonnet
- **Permissions**: ✅ File Read, ✅ File Write, ✅ Network, ❌ Sandbox
- **Description**: Automates Git workflow with intelligent commit messages following Conventional Commits specification
- **Default Task**: "Push all changes."

### 🛡️ Security Scanner
- **Icon**: 🛡️ `shield`
- **Model**: Opus
- **Permissions**: ✅ File Read, ✅ File Write, ❌ Network, ❌ Sandbox
- **Description**: Advanced AI-powered Static Application Security Testing (SAST) performing comprehensive security audits including:
  - Codebase intelligence gathering
  - Threat modeling (STRIDE)
  - Vulnerability scanning (OWASP Top 10, CWE)
  - Exploit validation
  - Remediation design
  - Professional report generation
- **Default Task**: "Review the codebase for security issues."

### 🧪 Unit Tests Bot
- **Icon**: 💻 `code`
- **Model**: Opus
- **Permissions**: ✅ File Read, ✅ File Write, ❌ Network, ❌ Sandbox
- **Description**: Automated comprehensive unit test generation analyzing codebase structure, creating test plans, writing tests matching your style, verifying execution, and optimizing coverage (>80% overall, 100% critical paths)
- **Default Task**: "Generate unit tests for this codebase."

### Available Icons

When creating agents, choose from these icon options:
- `bot` - 🤖 General purpose
- `shield` - 🛡️ Security related
- `code` - 💻 Development
- `terminal` - 🖥️ System/CLI
- `database` - 🗄️ Data operations
- `globe` - 🌐 Network/Web
- `file-text` - 📄 Documentation
- `git-branch` - 🌿 Version control

## Using Agents

### Importing Agents

#### Method 1: Import from GitHub (Recommended)
1. Navigate to **CC Agents** in Claudia
2. Click the **Import** dropdown button
3. Select **From GitHub**
4. Browse available agents from the official repository
5. Preview agent details and click **Import Agent**

#### Method 2: Import from Local File
1. Download a `.claudia.json` file from the repository
2. Navigate to **CC Agents** in Claudia
3. Click the **Import** dropdown button
4. Select **From File**
5. Choose the downloaded `.claudia.json` file

### Exporting Agents

1. Navigate to **CC Agents** in Claudia
2. Find your agent in the grid
3. Click the **Export** button
4. Choose where to save the `.claudia.json` file

### Executing Agents

1. Navigate to **CC Agents**
2. Click on an agent card to select it
3. Click **Execute Agent** or use the play button
4. Enter your task (or use the default)
5. Select the project directory
6. Choose the model (if different from agent default)
7. Click **Execute** to start

## Agent File Format

All agents are stored in `.claudia.json` format:

```json
{
  "version": 1,
  "exported_at": "2025-01-23T14:29:58.156063+00:00",
  "agent": {
    "name": "Your Agent Name",
    "icon": "bot",
    "model": "opus|sonnet|haiku",
    "system_prompt": "Your agent's instructions...",
    "default_task": "Default task description",
    "sandbox_enabled": false,
    "enable_file_read": true,
    "enable_file_write": true,
    "enable_network": false
  }
}
```

### Field Descriptions

- **version**: Schema version for forward compatibility
- **exported_at**: ISO 8601 timestamp of export
- **agent**: Agent configuration object
  - **name**: Display name for the agent
  - **icon**: Icon identifier (see available icons above)
  - **model**: Claude model to use (opus, sonnet, or haiku)
  - **system_prompt**: Instructions that define agent behavior
  - **default_task**: Pre-filled task when executing
  - **sandbox_enabled**: Whether to run in sandboxed environment
  - **enable_file_read**: Allow reading project files
  - **enable_file_write**: Allow modifying project files
  - **enable_network**: Allow internet access

## Technical Implementation

### Architecture Overview

The agent system consists of several integrated components:

#### Frontend Components
- **CCAgents.tsx**: Main agent management interface
- **GitHubAgentBrowser.tsx**: GitHub repository browser
- **CreateAgent.tsx**: Agent creation/editing form
- **AgentExecution.tsx**: Agent execution interface

#### Backend Modules

The backend agent system is implemented in Rust with a clean architecture:

```
src-tauri/src/commands/agents/
├── mod.rs         # Main module with Tauri commands
├── error.rs       # Custom error types
├── constants.rs   # Pricing and configuration
├── types.rs       # Domain types and validation
├── helpers.rs     # Utility functions
├── execute.rs     # Agent execution logic
├── repository.rs  # Database operations
├── service.rs     # Business logic layer
├── pool.rs        # Connection pooling
└── commands.rs    # Command interface
```

### Key Implementation Details

#### Type Safety
The system uses strong typing with newtype wrappers:
```rust
pub struct AgentId(i64);
pub struct RunId(i64);
pub struct SessionId(String);
```

#### Error Handling
Custom error types provide detailed error information:
```rust
pub enum AgentError {
    AgentNotFound(i64),
    RunNotFound(i64),
    InvalidStatus(String),
    Database(String),
    // ... more variants
}
```

#### Process Management
- Agents run as separate processes using Claude Code CLI
- Output is streamed in real-time via JSONL
- Sessions are isolated to prevent cross-contamination
- Graceful shutdown with timeout fallbacks

#### Security Model
- Configurable sandbox profiles per agent
- Platform-specific sandboxing (macOS, Linux, Windows)
- Violation tracking without blocking execution
- Essential system paths always accessible

### Database Schema

Agents are stored in SQLite with the following structure:

```sql
CREATE TABLE agents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    icon TEXT NOT NULL,
    system_prompt TEXT NOT NULL,
    default_task TEXT,
    model TEXT NOT NULL DEFAULT 'sonnet',
    sandbox_enabled BOOLEAN NOT NULL DEFAULT 0,
    enable_file_read BOOLEAN NOT NULL DEFAULT 1,
    enable_file_write BOOLEAN NOT NULL DEFAULT 1,
    enable_network BOOLEAN NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE agent_runs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    agent_id INTEGER NOT NULL,
    agent_name TEXT NOT NULL,
    agent_icon TEXT NOT NULL,
    task TEXT NOT NULL,
    model TEXT NOT NULL,
    project_path TEXT NOT NULL,
    session_id TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    pid INTEGER,
    process_started_at TEXT,
    scheduled_start_time TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at TEXT,
    usage_limit_reset_time TEXT,
    auto_resume_enabled BOOLEAN DEFAULT 0,
    resume_count INTEGER DEFAULT 0,
    parent_run_id INTEGER,
    FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE
);
```

### Import/Export Implementation

#### Export Process
1. Agent data is fetched from database
2. Serialized to JSON with version metadata
3. Saved to user-specified location via file dialog

#### Import Process
1. JSON file is parsed and validated
2. Duplicate names are handled with numeric suffixes
3. Agent is inserted into database
4. UI is notified to refresh

#### GitHub Integration
1. Fetches agent list from GitHub API
2. Downloads individual agent files on demand
3. Validates and imports using standard process

## Creating Custom Agents

### Step 1: Design Your Agent

Consider:
- **Purpose**: What specific task will it perform?
- **Model**: Which Claude model is appropriate?
- **Permissions**: What access does it need?
- **Instructions**: Clear, comprehensive system prompt

### Step 2: Create in Claudia

1. Navigate to **CC Agents**
2. Click **Create Agent**
3. Fill in the form:
   - **Name**: Descriptive, unique name
   - **Icon**: Choose appropriate icon
   - **System Prompt**: Detailed instructions
   - **Default Task**: Common use case
   - **Model**: Select based on complexity
   - **Permissions**: Enable only what's needed

### Step 3: Test Your Agent

1. Execute with various tasks
2. Verify output quality
3. Check permission requirements
4. Refine system prompt as needed

### Best Practices

#### System Prompt Guidelines
- Start with agent's role and expertise
- Define specific behaviors and constraints
- Include output format requirements
- Add examples if helpful
- Specify error handling approach

#### Permission Guidelines
- **File Read**: Usually needed for code analysis
- **File Write**: Required for code generation/modification
- **Network**: Only for API calls or web access
- **Sandbox**: Disable for trusted operations

#### Model Selection
- **Haiku**: Simple, fast tasks (formatting, basic analysis)
- **Sonnet**: General purpose (most agents)
- **Opus**: Complex reasoning (security analysis, architecture)

## Contributing Agents

### Contribution Process

1. **Create Your Agent**: Design and test thoroughly
2. **Export**: Save as descriptive `.claudia.json` file
3. **Fork Repository**: Fork the Claudia repository
4. **Add Agent**: Place file in `cc_agents` directory
5. **Update README**: Add agent to the table
6. **Submit PR**: Include description of functionality

### Contribution Guidelines

#### Agent Requirements
- **Single Purpose**: Focus on one specific task
- **Well Documented**: Clear system prompt
- **Tested**: Verify on multiple projects
- **Safe Defaults**: Conservative permissions

#### README Update Format
```markdown
| **Agent Name**<br/>🎯 `icon` | <img src="https://img.shields.io/badge/Model-color?style=flat-square" alt="Model"> | ✅/❌ Permissions | **Brief description**<br/><br/>Detailed description... | "Default task" |
```

#### Pull Request Template
```markdown
## New Agent: [Agent Name]

### Description
[What does this agent do?]

### Use Cases
- [Primary use case]
- [Secondary use case]

### Testing
- [ ] Tested on multiple projects
- [ ] Verified permission requirements
- [ ] Confirmed model selection

### Notes
[Any additional information]
```

### Review Process

Submitted agents are reviewed for:
1. **Functionality**: Does it work as described?
2. **Security**: Are permissions appropriate?
3. **Quality**: Is the output high quality?
4. **Documentation**: Is it well documented?
5. **Uniqueness**: Does it add value?

## Advanced Topics

### Scheduled Execution
Agents can be scheduled for future execution:
1. Select agent and configure task
2. Choose "Schedule" instead of "Execute"
3. Set date and time
4. Agent will run automatically

### Batch Operations
Execute agents on multiple projects:
1. Select agent
2. Choose multiple project directories
3. Agent runs sequentially on each

### Custom Workflows
Chain agents for complex workflows:
1. Use one agent's output as another's input
2. Schedule sequential executions
3. Monitor progress in Running Sessions

### Performance Optimization
- Use appropriate models for task complexity
- Disable unnecessary permissions
- Leverage caching for repeated operations
- Monitor token usage in metrics

## Troubleshooting

### Common Issues

#### Import Fails
- Verify JSON format is valid
- Check version compatibility
- Ensure unique agent name

#### Execution Errors
- Verify Claude Code CLI is installed
- Check permission requirements
- Review system prompt for clarity

#### Performance Issues
- Consider using lighter model
- Reduce task complexity
- Check system resources

### Debug Mode
Enable debug logging:
1. Open Developer Tools (F12)
2. Check Console for detailed logs
3. Review Network tab for API calls

## Future Enhancements

### Planned Features
1. **Agent Marketplace**: Browse community agents
2. **Agent Composition**: Combine multiple agents
3. **Visual Agent Builder**: Drag-and-drop creation
4. **Agent Analytics**: Usage statistics and insights
5. **Version History**: Track agent changes
6. **Collaborative Editing**: Team agent development

### API Extensions
- Webhook support for external triggers
- REST API for programmatic access
- Plugin system for custom capabilities
- Integration with CI/CD pipelines

## Resources

- [Claude Code Documentation](https://www.claude.ai/docs)
- [Agent Examples Repository](https://github.com/yourusername/claudia/tree/main/cc_agents)
- [Community Forums](https://community.claudia.app)
- [Video Tutorials](https://youtube.com/claudia-agents)