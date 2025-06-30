import React from "react";
import { Search } from "lucide-react";
import { cn } from "@/lib/utils";

interface GlobWidgetProps {
  pattern: string;
  result?: any;
}

const GlobWidgetComponent: React.FC<GlobWidgetProps> = ({ pattern, result }) => {
  // Extract result content if available
  let resultContent = '';
  let isError = false;
  
  if (result) {
    isError = result.is_error || false;
    if (typeof result.content === 'string') {
      resultContent = result.content;
    } else if (result.content && typeof result.content === 'object') {
      if (result.content.text) {
        resultContent = result.content.text;
      } else if (Array.isArray(result.content)) {
        resultContent = result.content
          .map((c: any) => (typeof c === 'string' ? c : c.text || JSON.stringify(c)))
          .join('\n');
      } else {
        resultContent = JSON.stringify(result.content, null, 2);
      }
    }
  }
  
  return (
    <div className="space-y-2">
      <div className="flex items-center gap-2 p-3 rounded-lg bg-muted/50">
        <Search className="h-4 w-4 text-primary" />
        <span className="text-sm">Searching for pattern:</span>
        <code className="text-sm font-mono bg-background px-2 py-0.5 rounded">
          {pattern}
        </code>
        {!result && (
          <div className="ml-auto flex items-center gap-1 text-xs text-muted-foreground">
            <div className="h-2 w-2 bg-blue-500 rounded-full animate-pulse" />
            <span>Searching...</span>
          </div>
        )}
      </div>
      
      {/* Show result if available */}
      {result && (
        <div className={cn(
          "p-3 rounded-md border text-xs font-mono whitespace-pre-wrap overflow-x-auto",
          isError 
            ? "border-red-500/20 bg-red-500/5 text-red-400" 
            : "border-green-500/20 bg-green-500/5 text-green-300"
        )}>
          {resultContent || (isError ? "Search failed" : "No matches found")}
        </div>
      )}
    </div>
  );
};

export const GlobWidget = React.memo(GlobWidgetComponent);