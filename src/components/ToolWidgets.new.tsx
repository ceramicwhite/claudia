import React from "react";
import { WidgetFactory } from "./widgets";

// Re-export all the original widgets for backward compatibility
export * from "./widgets";

// Create a wrapper component that demonstrates how to use the WidgetFactory
export const ToolWidget: React.FC<{
  toolName: string;
  params: any;
  result?: any;
}> = ({ toolName, params, result }) => {
  return <WidgetFactory toolName={toolName} params={params} result={result} />;
};

// Example of how to use the new structure in your application:
/*
import { ToolWidget } from "@/components/ToolWidgets";

// In your component:
<ToolWidget 
  toolName="TodoWrite" 
  params={{ todos: [...] }} 
  result={result}
/>

// Or use specific widgets directly:
import { TodoWidget, ReadWidget } from "@/components/widgets";

<TodoWidget todos={todos} />
<ReadWidget filePath="/path/to/file" result={result} />
*/