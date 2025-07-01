# Widget Components

This directory contains the refactored widget components extracted from the original ToolWidgets.tsx file.

## Structure

The widgets are organized by functionality:

- **common/** - Shared components used across multiple widgets
- **todo/** - Todo list management widgets
- **file/** - File operation widgets (LS, Read, Write, Edit, MultiEdit)
- **search/** - Search widgets (Grep, Glob)
- **command/** - Command execution widgets (Bash)
- **system/** - System message widgets
- **mcp/** - Model Context Protocol widgets (to be extracted)
- **task/** - Task management widgets (to be extracted)
- **web/** - Web search widgets (to be extracted)
- **thinking/** - Thinking/reasoning widgets (to be extracted)

## Usage

### Import individual widgets

```tsx
import { TodoWidget } from "@/components/widgets/todo/TodoWidget";
import { ReadWidget } from "@/components/widgets/file/ReadWidget";
```

### Import from main index

```tsx
import { TodoWidget, ReadWidget, GrepWidget } from "@/components/widgets";
```

### Use WidgetFactory for dynamic rendering

```tsx
import { WidgetFactory } from "@/components/widgets";

// Dynamically render a widget based on tool name
<WidgetFactory 
  toolName="TodoWrite" 
  params={{ todos: [...] }} 
  result={result} 
/>
```

## Adding New Widgets

1. Create a new widget file in the appropriate directory
2. Follow the existing widget pattern:
   - Accept `params` and optional `result` props
   - Handle loading states when `result` is not available
   - Use common components for consistency
3. Export from the directory's index file
4. Register in WidgetFactory

## Common Components

### WidgetContainer
Provides a consistent container with expand/collapse functionality:

```tsx
<WidgetContainer 
  icon={MyIcon} 
  title="My Widget"
  defaultExpanded={true}
>
  {/* Widget content */}
</WidgetContainer>
```

### WidgetHeader
Simple header component for widgets:

```tsx
<WidgetHeader icon={FileEdit} title="Todo List" />
```

### WidgetError
Consistent error display:

```tsx
<WidgetError error="Something went wrong" />
```

## Widget Registration

Register custom widgets dynamically:

```tsx
import { registerWidget } from "@/components/widgets";

registerWidget("MyCustomTool", MyCustomWidget);
```

## Migration Status

### Completed
- ✅ TodoWidget
- ✅ LSWidget & LSResultWidget
- ✅ ReadWidget & ReadResultWidget
- ✅ WriteWidget
- ✅ EditWidget & EditResultWidget
- ✅ MultiEditWidget
- ✅ GrepWidget
- ✅ GlobWidget
- ✅ BashWidget
- ✅ SystemReminderWidget

### To Be Extracted
- ⏳ MCPWidget
- ⏳ CommandWidget & CommandOutputWidget
- ⏳ SummaryWidget
- ⏳ SystemInitializedWidget
- ⏳ TaskWidget
- ⏳ WebSearchWidget
- ⏳ ThinkingWidget
- ⏳ MultiEditResultWidget

## Benefits

1. **Modularity**: Each widget is self-contained
2. **Maintainability**: Easier to find and update specific widgets
3. **Reusability**: Common patterns extracted to shared components
4. **Extensibility**: Easy to add new widgets
5. **Type Safety**: Proper TypeScript interfaces for each widget
6. **Performance**: Smaller file sizes, better code splitting