# Widget System Documentation

This section documents Claudia's widget system for rendering AI tool outputs and interactions.

## Contents

### Overview
- [Widget Architecture](./architecture.md) - System design and concepts
- [Widget Types](./types.md) - Available widget categories
- [Widget Factory](./factory.md) - Dynamic widget creation

### Widget Components

#### File Operations
- [Read Widget](./file/read-widget.md) - File reading display
- [Edit Widget](./file/edit-widget.md) - File editing interface
- [Write Widget](./file/write-widget.md) - File creation display
- [LS Widget](./file/ls-widget.md) - Directory listing

#### Search & Discovery
- [Grep Widget](./search/grep-widget.md) - Content search results
- [Glob Widget](./search/glob-widget.md) - File pattern matching

#### Task Management
- [Todo Widget](./todo/todo-widget.md) - Task list management
- [Task Planning](./task/planning-widget.md) - AI task planning

#### System
- [System Reminder](./system/reminder-widget.md) - Context reminders
- [Thinking Widget](./thinking/thinking-widget.md) - AI reasoning display

#### Commands
- [Bash Widget](./command/bash-widget.md) - Command execution display

#### Web Operations
- [Web Fetch](./web/fetch-widget.md) - Web content display
- [Web Search](./web/search-widget.md) - Search results

### Development
- [Creating Widgets](./creating-widgets.md) - Widget development guide
- [Widget API](./widget-api.md) - Component interfaces
- [Migration Guide](./MIGRATION.md) - Upgrading widgets

## Widget System Overview

The widget system provides a flexible way to render different types of AI tool outputs in the Claudia interface. Each widget is designed to handle specific data types and provide optimal user experience.

### Core Concepts

1. **Widget Factory**: Dynamically creates widgets based on tool names
2. **Widget Container**: Provides consistent wrapper with headers
3. **Widget Props**: Standardized interface for all widgets
4. **Error Handling**: Built-in error display for all widgets

### Widget Categories

- **File Operations**: Display file contents, edits, and directory listings
- **Search Tools**: Show search and pattern matching results
- **Task Management**: Handle todo lists and planning
- **System Tools**: Display system information and AI reasoning
- **Command Execution**: Show command outputs and results
- **Web Tools**: Display web content and search results

## Quick Start

### Using Widgets

```typescript
import { WidgetFactory } from '@/components/widgets';

// In your component
<WidgetFactory
  toolName="read_file"
  args={{ path: "/path/to/file.txt" }}
  content="File contents here..."
  isLoading={false}
/>
```

### Creating a Widget

```typescript
import { WidgetContainer } from '../common/WidgetContainer';

export const MyWidget: React.FC<WidgetProps> = ({ args, content }) => {
  return (
    <WidgetContainer title="My Widget">
      {/* Widget content */}
    </WidgetContainer>
  );
};
```

## Best Practices

1. **Consistent UI**: Use WidgetContainer for consistent appearance
2. **Error Handling**: Always handle error states gracefully
3. **Loading States**: Show appropriate loading indicators
4. **Accessibility**: Ensure widgets are keyboard navigable
5. **Performance**: Optimize for large content rendering