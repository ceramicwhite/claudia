import React from "react";
import { WidgetFactory, TodoWidget, ReadWidget, hasWidget } from "@/components/widgets";

// Example 1: Using WidgetFactory for dynamic rendering
export const DynamicWidgetExample: React.FC = () => {
  const toolCalls = [
    {
      toolName: "TodoWrite",
      params: {
        todos: [
          { id: "1", content: "Complete refactoring", status: "in_progress", priority: "high" },
          { id: "2", content: "Write tests", status: "pending", priority: "medium" }
        ]
      },
      result: { success: true }
    },
    {
      toolName: "Read",
      params: { filePath: "/src/components/Example.tsx" },
      result: { content: "// File content here..." }
    }
  ];

  return (
    <div className="space-y-4">
      {toolCalls.map((call, index) => (
        <WidgetFactory
          key={index}
          toolName={call.toolName}
          params={call.params}
          result={call.result}
        />
      ))}
    </div>
  );
};

// Example 2: Using specific widgets directly
export const DirectWidgetExample: React.FC = () => {
  const todos = [
    { id: "1", content: "Implement feature", status: "completed", priority: "high" },
    { id: "2", content: "Review PR", status: "in_progress", priority: "medium" }
  ];

  return (
    <div className="space-y-4">
      <TodoWidget todos={todos} />
      <ReadWidget filePath="/path/to/file.ts" />
    </div>
  );
};

// Example 3: Conditional rendering based on tool availability
export const ConditionalWidgetExample: React.FC<{ toolName: string; params: any }> = ({ 
  toolName, 
  params 
}) => {
  // Check if widget is registered
  if (!hasWidget(toolName)) {
    return (
      <div className="p-4 border rounded-lg bg-muted">
        <p className="text-sm text-muted-foreground">
          Widget for "{toolName}" is not available
        </p>
      </div>
    );
  }

  return <WidgetFactory toolName={toolName} params={params} />;
};

// Example 4: Custom widget wrapper with error handling
export const SafeWidgetWrapper: React.FC<{
  toolName: string;
  params: any;
  result?: any;
}> = ({ toolName, params, result }) => {
  try {
    return (
      <div className="widget-wrapper">
        <WidgetFactory 
          toolName={toolName} 
          params={params} 
          result={result} 
        />
      </div>
    );
  } catch (error) {
    return (
      <div className="p-4 border border-destructive rounded-lg">
        <p className="text-sm text-destructive">
          Error rendering widget: {error instanceof Error ? error.message : "Unknown error"}
        </p>
      </div>
    );
  }
};

// Example 5: Tool execution with widget display
export const ToolExecutionExample: React.FC = () => {
  const [toolResults, setToolResults] = React.useState<Map<string, any>>(new Map());

  const executeTool = async (toolName: string, params: any) => {
    // Simulate tool execution
    const result = await simulateToolExecution(toolName, params);
    setToolResults(prev => new Map(prev).set(`${toolName}-${Date.now()}`, {
      toolName,
      params,
      result
    }));
  };

  return (
    <div className="space-y-4">
      <div className="flex gap-2">
        <button 
          onClick={() => executeTool("TodoWrite", { todos: [] })}
          className="px-4 py-2 bg-primary text-primary-foreground rounded"
        >
          Create Todo
        </button>
        <button 
          onClick={() => executeTool("Bash", { command: "ls -la" })}
          className="px-4 py-2 bg-primary text-primary-foreground rounded"
        >
          Run Command
        </button>
      </div>

      <div className="space-y-4">
        {Array.from(toolResults.entries()).map(([key, data]) => (
          <WidgetFactory
            key={key}
            toolName={data.toolName}
            params={data.params}
            result={data.result}
          />
        ))}
      </div>
    </div>
  );
};

// Helper function to simulate tool execution
async function simulateToolExecution(toolName: string, params: any) {
  // Simulate async operation
  await new Promise(resolve => setTimeout(resolve, 1000));
  
  // Return mock results based on tool
  switch (toolName) {
    case "TodoWrite":
      return { success: true, message: "Todos updated" };
    case "Bash":
      return { content: "Command output here..." };
    default:
      return { success: true };
  }
}