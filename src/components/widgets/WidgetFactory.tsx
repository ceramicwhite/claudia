import React from "react";
import type { WidgetProps, BaseWidgetProps } from "./types";

// Widget imports
import { TodoWidget } from "./todo/TodoWidget";
import { LSWidget } from "./file/LSWidget";
import { LSResultWidget } from "./file/LSResultWidget";
import { ReadWidget } from "./file/ReadWidget";
import { ReadResultWidget } from "./file/ReadResultWidget";
import { WriteWidget } from "./file/WriteWidget";
import { EditWidget } from "./file/EditWidget";
import { EditResultWidget } from "./file/EditResultWidget";
import { MultiEditWidget } from "./file/MultiEditWidget";
import { GrepWidget } from "./search/GrepWidget";
import { GlobWidget } from "./search/GlobWidget";
import { BashWidget } from "./command/BashWidget";
import { SystemReminderWidget } from "./system/SystemReminderWidget";

// Widget mapping type - components must accept BaseWidgetProps at minimum
type WidgetComponent<T extends BaseWidgetProps = BaseWidgetProps> = React.FC<T>;

// Widget registry
const widgetRegistry: Record<string, WidgetComponent> = {
  // Todo widgets
  "TodoWrite": TodoWidget,
  "TodoRead": TodoWidget,
  
  // File operation widgets
  "LS": LSWidget,
  "LSResult": LSResultWidget,
  "Read": ReadWidget,
  "ReadResult": ReadResultWidget,
  "Write": WriteWidget,
  "Edit": EditWidget,
  "EditResult": EditResultWidget,
  "MultiEdit": MultiEditWidget,
  
  // Search widgets
  "Grep": GrepWidget,
  "Glob": GlobWidget,
  
  // Command widgets
  "Bash": BashWidget,
  
  // System widgets
  "SystemReminder": SystemReminderWidget,
  
  // Note: Add more widgets here as they are extracted
  // "MCP": MCPWidget,
  // "Command": CommandWidget,
  // "WebSearch": WebSearchWidget,
  // "Task": TaskWidget,
  // "Thinking": ThinkingWidget,
};

interface WidgetFactoryProps extends BaseWidgetProps {
  toolName: string;
  params?: Record<string, unknown>;
  result?: unknown;
}

export const WidgetFactory: React.FC<WidgetFactoryProps> = ({ 
  toolName, 
  params = {},
  result 
}) => {
  // Find the appropriate widget component
  const WidgetComponent = widgetRegistry[toolName];
  
  if (!WidgetComponent) {
    // Return a default widget for unregistered tools
    return (
      <div className="p-3 rounded-lg bg-muted/50 text-sm">
        <span className="font-medium">{toolName}</span>
        {params && (
          <pre className="mt-2 text-xs font-mono opacity-75">
            {JSON.stringify(params, null, 2)}
          </pre>
        )}
      </div>
    );
  }
  
  // Merge params with base props
  const widgetProps: WidgetProps = {
    toolName,
    ...params,
    result,
  } as WidgetProps;
  
  // Render the appropriate widget with its props
  return <WidgetComponent {...widgetProps} />;
};

// Export function to register new widgets dynamically
export const registerWidget = <T extends BaseWidgetProps = BaseWidgetProps>(
  toolName: string, 
  component: WidgetComponent<T>
) => {
  widgetRegistry[toolName] = component as WidgetComponent;
};

// Export function to check if a widget is registered
export const hasWidget = (toolName: string): boolean => {
  return toolName in widgetRegistry;
};