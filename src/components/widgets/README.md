# Widget Components

This folder contains all the widget components used to render tool outputs in the Claude Code UI.

## Structure

```
widgets/
├── WidgetFactory.tsx     # Main factory for dynamic widget selection
├── index.ts              # Central export file
├── types.ts              # TypeScript types and interfaces
├── command/              # Command-related widgets
│   ├── BashWidget.tsx
│   ├── CommandWidget.tsx
│   └── CommandOutputWidget.tsx
├── file/                 # File operation widgets
│   ├── EditWidget.tsx
│   ├── EditResultWidget.tsx
│   ├── LSWidget.tsx
│   ├── LSResultWidget.tsx
│   ├── MultiEditWidget.tsx
│   ├── MultiEditResultWidget.tsx
│   ├── ReadWidget.tsx
│   ├── ReadResultWidget.tsx
│   ├── WriteWidget.tsx
│   └── utils.ts
├── mcp/                  # Model Context Protocol widgets
│   └── MCPWidget.tsx
├── search/               # Search-related widgets
│   ├── GlobWidget.tsx
│   └── GrepWidget.tsx
├── system/               # System message widgets
│   ├── SystemInitializedWidget.tsx
│   ├── SystemReminderWidget.tsx
│   └── SummaryWidget.tsx
├── task/                 # Task/agent widgets
│   └── TaskWidget.tsx
├── thinking/             # AI thinking/reasoning widgets
│   └── ThinkingWidget.tsx
├── todo/                 # Todo list widgets
│   ├── TodoWidget.tsx
│   └── TodoItem.tsx
└── web/                  # Web-related widgets
    └── WebSearchWidget.tsx
```

## Usage

### Using the WidgetFactory

For dynamic widget selection based on tool name:

```tsx
import { WidgetFactory } from "@/components/widgets";

<WidgetFactory 
  toolName="TodoWrite" 
  params={{ todos: [...] }} 
  result={result}
/>
```

### Direct Widget Import

For specific widgets when you know the type:

```tsx
import { TodoWidget, ReadWidget } from "@/components/widgets";

<TodoWidget todos={todos} />
<ReadWidget filePath="/path/to/file" result={result} />
```

### Registering New Widgets

To add a new widget:

1. Create the widget component in the appropriate folder
2. Export it from the folder's index file
3. Register it in `WidgetFactory.tsx`:

```tsx
import { MyNewWidget } from "./my-category/MyNewWidget";

const widgetRegistry: Record<string, WidgetComponent> = {
  // ... existing widgets
  "MyNewTool": MyNewWidget,
};
```

## Widget Props

All widgets receive a common set of props defined in `types.ts`:

- `toolName`: The name of the tool
- `result`: Optional result data from tool execution
- Additional tool-specific parameters

## Migration from ToolWidgets.tsx

The old monolithic `ToolWidgets.tsx` file has been split into this modular structure. To migrate:

1. Update imports from `./ToolWidgets` to `./widgets`
2. Use the same widget names - they're all exported
3. Consider using the WidgetFactory for dynamic selection

## Adding New Widgets

When creating a new widget:

1. Place it in the appropriate category folder
2. Follow the naming convention: `[ToolName]Widget.tsx`
3. Export it from the category's index file
4. Register it in the WidgetFactory
5. Add appropriate TypeScript types
6. Document any special props or behavior