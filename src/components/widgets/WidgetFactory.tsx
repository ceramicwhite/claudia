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
import { MultiEditResultWidget } from "./file/MultiEditResultWidget";
import { GrepWidget } from "./search/GrepWidget";
import { GlobWidget } from "./search/GlobWidget";
import { BashWidget } from "./command/BashWidget";
import { CommandWidget } from "./command/CommandWidget";
import { CommandOutputWidget } from "./command/CommandOutputWidget";
import { SystemReminderWidget } from "./system/SystemReminderWidget";
import { SystemInitializedWidget } from "./system/SystemInitializedWidget";
import { SummaryWidget } from "./system/SummaryWidget";
import { MCPWidget } from "./mcp/MCPWidget";
import { TaskWidget } from "./task/TaskWidget";
import { WebSearchWidget } from "./web/WebSearchWidget";
import { ThinkingWidget } from "./thinking/ThinkingWidget";

// Widget mapping type - any React component
type WidgetComponent = React.FC<any>;

// Widget registry - maps tool names to their components
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
  "MultiEditResult": MultiEditResultWidget,
  
  // Search widgets
  "Grep": GrepWidget,
  "Glob": GlobWidget,
  
  // Command widgets
  "Bash": BashWidget,
  "Command": CommandWidget,
  "CommandOutput": CommandOutputWidget,
  
  // System widgets
  "SystemReminder": SystemReminderWidget,
  "SystemInitialized": SystemInitializedWidget,
  "Summary": SummaryWidget,
  
  // MCP widget
  "MCP": MCPWidget,
  
  // Task widget
  "Task": TaskWidget,
  
  // Web widgets
  "WebSearch": WebSearchWidget,
  
  // Thinking widget
  "Thinking": ThinkingWidget,
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
export const registerWidget = (
  toolName: string, 
  component: WidgetComponent
) => {
  widgetRegistry[toolName] = component;
};

// Export function to check if a widget is registered
export const hasWidget = (toolName: string): boolean => {
  return toolName in widgetRegistry;
};