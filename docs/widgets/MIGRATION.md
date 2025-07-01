# Widget Refactoring Migration Guide

## Overview

The ToolWidgets.tsx file has been split into smaller, focused widget components organized by functionality.

## New Structure

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
│   ├── LSWidget.tsx
│   ├── LSResultWidget.tsx
│   ├── ReadWidget.tsx
│   ├── ReadResultWidget.tsx
│   ├── WriteWidget.tsx
│   ├── EditWidget.tsx
│   ├── EditResultWidget.tsx
│   ├── MultiEditWidget.tsx
│   └── utils.ts            # Shared utilities (getLanguageFromPath)
├── search/                  # Search widgets
│   ├── GrepWidget.tsx
│   └── GlobWidget.tsx
├── command/                 # Command execution widgets
│   └── BashWidget.tsx
├── WidgetFactory.tsx        # Dynamic widget rendering
└── index.ts                 # Main exports
```

## Migration Steps

### 1. Update imports in components using widgets

**Before:**
```tsx
import { TodoWidget, ReadWidget } from "@/components/ToolWidgets";
```

**After:**
```tsx
// Option 1: Import from the new structure
import { TodoWidget, ReadWidget } from "@/components/widgets";

// Option 2: Import specific widgets
import { TodoWidget } from "@/components/widgets/todo/TodoWidget";
import { ReadWidget } from "@/components/widgets/file/ReadWidget";
```

### 2. Use WidgetFactory for dynamic rendering

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
  return <WidgetFactory toolName={toolName} params={params} result={result} />;
};
```

### 3. Register custom widgets

```tsx
import { registerWidget } from "@/components/widgets";
import { CustomWidget } from "./CustomWidget";

// Register a new widget
registerWidget("CustomTool", CustomWidget);
```

### 4. Use common components for consistency

```tsx
import { WidgetContainer, WidgetHeader, WidgetError } from "@/components/widgets";

export const MyWidget = ({ data, error }) => {
  return (
    <WidgetContainer icon={MyIcon} title="My Widget">
      {error ? (
        <WidgetError error={error} />
      ) : (
        <div>{/* Widget content */}</div>
      )}
    </WidgetContainer>
  );
};
```

## Benefits

1. **Better organization**: Widgets are grouped by functionality
2. **Easier maintenance**: Each widget is in its own file
3. **Reusable components**: Common patterns extracted to shared components
4. **Dynamic rendering**: WidgetFactory handles tool-to-widget mapping
5. **Extensibility**: Easy to add new widgets via registration

## Adding New Widgets

1. Create a new widget file in the appropriate directory
2. Export it from the directory's index file
3. Register it in WidgetFactory or use registerWidget()
4. The widget will automatically be available for use

## Backward Compatibility

The original exports are maintained through the index.ts file, so existing code should continue to work without changes. However, we recommend migrating to the new structure for better maintainability.