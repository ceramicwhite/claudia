# Widget System Documentation

The widget system provides a flexible and extensible way to render different types of AI tool outputs in the Claudia interface. Each widget is designed to handle specific data types and provide an optimal user experience.

## Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Widget Types](#widget-types)
- [Using Widgets](#using-widgets)
- [Creating New Widgets](#creating-new-widgets)
- [Migration Guide](#migration-guide)
- [API Reference](#api-reference)

## Overview

The widget system is built around the concept of specialized components that render tool outputs from Claude's interactions. Each tool (like file reading, web searching, or task management) has a corresponding widget that knows how to best display its results.

### Core Concepts

1. **Widget Factory**: Dynamically creates widgets based on tool names
2. **Widget Container**: Provides consistent wrapper with headers and controls
3. **Widget Props**: Standardized interface for all widgets
4. **Error Handling**: Built-in error display for all widgets
5. **Extensibility**: Easy registration of new widgets

### Benefits

- **Consistency**: All widgets share common UI patterns
- **Modularity**: Each widget is self-contained and focused
- **Reusability**: Common components reduce duplication
- **Maintainability**: Organized structure makes updates easier
- **Type Safety**: Full TypeScript support throughout

## Architecture

### Directory Structure

```
src/components/widgets/
├── common/                  # Shared components
│   ├── WidgetContainer.tsx  # Common container with expand/collapse
│   ├── WidgetHeader.tsx     # Common header component
│   └── WidgetError.tsx      # Error display component
├── todo/                    # Todo-related widgets
│   ├── TodoWidget.tsx
│   └── TodoItem.tsx
├── file/                    # File operation widgets
│   ├── LSWidget.tsx         # Directory listing
│   ├── LSResultWidget.tsx   # LS operation results
│   ├── ReadWidget.tsx       # File reading
│   ├── ReadResultWidget.tsx # Read operation results
│   ├── WriteWidget.tsx      # File writing
│   ├── EditWidget.tsx       # File editing
│   ├── EditResultWidget.tsx # Edit operation results
│   ├── MultiEditWidget.tsx  # Multiple file edits
│   └── utils.ts            # Shared utilities
├── search/                  # Search widgets
│   ├── GrepWidget.tsx       # Content search
│   └── GlobWidget.tsx       # File pattern search
├── command/                 # Command execution widgets
│   └── BashWidget.tsx       # Bash command execution
├── system/                  # System widgets
│   └── SystemReminderWidget.tsx # System reminders
├── thinking/                # AI reasoning widgets
│   └── ThinkingWidget.tsx   # Thinking process display
├── task/                    # Task planning widgets
├── web/                     # Web operation widgets
├── mcp/                     # Model Context Protocol widgets
├── WidgetFactory.tsx        # Dynamic widget rendering
├── types.ts                 # TypeScript types
└── index.ts                 # Main exports
```

### Component Hierarchy

```
WidgetFactory
  └── WidgetContainer
        ├── WidgetHeader
        │     ├── Icon
        │     ├── Title
        │     └── Controls (expand/collapse)
        └── Widget Content
              └── Specific Widget Component
```

## Widget Types

### File Operations

#### Read Widget
Displays file contents with syntax highlighting and line numbers.

```typescript
<ReadWidget 
  args={{ path: "/path/to/file.txt" }}
  content="File contents here..."
/>
```

#### Edit Widget
Shows file modifications with diff view.

```typescript
<EditWidget
  args={{
    file_path: "/path/to/file.txt",
    old_string: "original",
    new_string: "modified"
  }}
  content="Edit successful"
/>
```

#### Write Widget
Displays newly created files.

```typescript
<WriteWidget
  args={{
    file_path: "/path/to/new-file.txt",
    content: "New file content"
  }}
/>
```

#### LS Widget
Shows directory listings in a structured format.

```typescript
<LSWidget
  args={{ path: "/path/to/directory" }}
  content={lsOutput}
/>
```

### Search & Discovery

#### Grep Widget
Displays content search results with matched lines highlighted.

```typescript
<GrepWidget
  args={{
    pattern: "searchTerm",
    path: "/search/path"
  }}
  content={searchResults}
/>
```

#### Glob Widget
Shows file pattern matching results.

```typescript
<GlobWidget
  args={{
    pattern: "**/*.tsx",
    path: "/project/root"
  }}
  content={matchedFiles}
/>
```

### Task Management

#### Todo Widget
Interactive task list with add/remove/toggle functionality.

```typescript
<TodoWidget
  args={{ todos: todoArray }}
  onTodoChange={handleTodoUpdate}
/>
```

### System & AI

#### System Reminder Widget
Displays important system context or reminders.

```typescript
<SystemReminderWidget
  content="Important context information"
/>
```

#### Thinking Widget
Shows AI reasoning process step by step.

```typescript
<ThinkingWidget
  args={{
    thought: "Current reasoning step",
    thoughtNumber: 1,
    totalThoughts: 5
  }}
/>
```

### Command Execution

#### Bash Widget
Displays command execution with output formatting.

```typescript
<BashWidget
  args={{
    command: "ls -la",
    timeout: 30000
  }}
  content={commandOutput}
/>
```

## Using Widgets

### Basic Usage

The simplest way to use widgets is through the WidgetFactory:

```typescript
import { WidgetFactory } from '@/components/widgets';

function MyComponent() {
  return (
    <WidgetFactory
      toolName="read_file"
      args={{ path: "/example.txt" }}
      content="File contents..."
      isLoading={false}
      error={null}
    />
  );
}
```

### Direct Widget Import

For more control, import widgets directly:

```typescript
import { ReadWidget } from '@/components/widgets/file/ReadWidget';

function MyComponent() {
  return (
    <ReadWidget
      args={{ path: "/example.txt" }}
      content="File contents..."
      isLoading={false}
    />
  );
}
```

### With Error Handling

```typescript
<WidgetFactory
  toolName="read_file"
  args={{ path: "/example.txt" }}
  content={null}
  error="File not found"
  isLoading={false}
/>
```

### Loading States

```typescript
<WidgetFactory
  toolName="bash"
  args={{ command: "long-running-command" }}
  content={null}
  isLoading={true}
/>
```

## Creating New Widgets

### Step 1: Create Widget Component

Create a new file in the appropriate category directory:

```typescript
// src/components/widgets/myCategory/MyWidget.tsx
import React from 'react';
import { WidgetProps } from '../types';
import { WidgetContainer } from '../common/WidgetContainer';
import { MyIcon } from 'lucide-react';

export const MyWidget: React.FC<WidgetProps> = ({ 
  args, 
  content, 
  isLoading, 
  error 
}) => {
  // Parse your specific args
  const { myParam } = args as { myParam: string };

  return (
    <WidgetContainer 
      icon={MyIcon} 
      title="My Widget"
      error={error}
      isLoading={isLoading}
    >
      <div className="p-4">
        {/* Your widget content */}
        <p>Parameter: {myParam}</p>
        <div>{content}</div>
      </div>
    </WidgetContainer>
  );
};
```

### Step 2: Register the Widget

Add your widget to the WidgetFactory:

```typescript
// src/components/widgets/WidgetFactory.tsx
import { MyWidget } from './myCategory/MyWidget';

const widgetMap: Record<string, React.ComponentType<WidgetProps>> = {
  // ... existing widgets
  'my_tool': MyWidget,
};
```

### Step 3: Export the Widget

Add to the category's index file:

```typescript
// src/components/widgets/myCategory/index.ts
export { MyWidget } from './MyWidget';
```

### Best Practices

1. **Use WidgetContainer**: Provides consistent UI and behavior
2. **Handle all states**: Loading, error, and success states
3. **Type your args**: Create interfaces for your widget's arguments
4. **Keep it focused**: Each widget should do one thing well
5. **Consider performance**: Use React.memo for expensive renders
6. **Add accessibility**: Ensure keyboard navigation and screen reader support

## Migration Guide

### From Monolithic ToolWidgets.tsx

The widget system was refactored from a single large file to a modular structure. Here's how to migrate:

#### Update Imports

**Before:**
```tsx
import { TodoWidget, ReadWidget } from "@/components/ToolWidgets";
```

**After:**
```tsx
// Option 1: Import from main exports
import { TodoWidget, ReadWidget } from "@/components/widgets";

// Option 2: Import from specific locations
import { TodoWidget } from "@/components/widgets/todo/TodoWidget";
import { ReadWidget } from "@/components/widgets/file/ReadWidget";
```

#### Use WidgetFactory for Dynamic Rendering

**Before:**
```tsx
const renderWidget = (toolName: string, params: any) => {
  switch(toolName) {
    case "TodoWrite":
      return <TodoWidget {...params} />;
    case "Read":
      return <ReadWidget {...params} />;
    // ... many more cases
  }
};
```

**After:**
```tsx
import { WidgetFactory } from "@/components/widgets";

const renderWidget = (toolName: string, params: any, result?: any) => {
  return <WidgetFactory 
    toolName={toolName} 
    args={params} 
    content={result} 
  />;
};
```

#### Register Custom Widgets

```tsx
import { registerWidget } from "@/components/widgets";
import { CustomWidget } from "./CustomWidget";

// Register at app initialization
registerWidget("custom_tool", CustomWidget);
```

### Benefits of Migration

1. **Better organization**: Widgets grouped by functionality
2. **Easier maintenance**: Each widget in its own file
3. **Reusable components**: Common patterns extracted
4. **Dynamic rendering**: Automatic tool-to-widget mapping
5. **Extensibility**: Easy to add new widgets

## API Reference

### WidgetProps Interface

```typescript
interface WidgetProps {
  args: Record<string, any>;      // Tool-specific arguments
  content?: string | null;        // Result content
  isLoading?: boolean;            // Loading state
  error?: string | null;          // Error message
  onUpdate?: (data: any) => void; // Optional update handler
}
```

### WidgetContainer Props

```typescript
interface WidgetContainerProps {
  icon?: React.ComponentType;     // Icon component
  title: string;                  // Widget title
  children: React.ReactNode;      // Widget content
  error?: string | null;          // Error to display
  isLoading?: boolean;            // Show loading state
  defaultExpanded?: boolean;      // Initial expand state
}
```

### Widget Registration

```typescript
function registerWidget(
  toolName: string, 
  component: React.ComponentType<WidgetProps>
): void;
```

### Common Utilities

```typescript
// Get language from file path for syntax highlighting
function getLanguageFromPath(path: string): string;

// Format file sizes
function formatFileSize(bytes: number): string;

// Parse tool arguments safely
function parseToolArgs<T>(args: any, defaults: T): T;
```

## Performance Considerations

1. **Lazy Loading**: Large widgets can be lazy-loaded
2. **Memoization**: Use React.memo for expensive renders
3. **Virtualization**: Long lists should use virtual scrolling
4. **Code Splitting**: Widget categories can be code-split

## Accessibility

All widgets should:
- Support keyboard navigation
- Have proper ARIA labels
- Announce state changes
- Provide focus indicators
- Support screen readers

## Future Enhancements

- Widget composition API
- Custom widget themes
- Widget state persistence
- Collaborative widgets
- Real-time update streaming
- Widget marketplace