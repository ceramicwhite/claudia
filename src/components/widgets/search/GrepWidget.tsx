import React, { useState } from "react";
import { Search, Code, FolderOpen, FilePlus, X, ChevronDown, ChevronRight, AlertCircle, FileText } from "lucide-react";
import { cn } from "@/lib/utils";
import { Badge } from "@/components/ui/badge";
import { Prism as SyntaxHighlighter } from "react-syntax-highlighter";
import { claudeSyntaxTheme } from "@/lib/claudeSyntaxTheme";
import { getLanguageFromPath } from "../file/utils";

interface GrepWidgetProps {
  pattern: string;
  include?: string;
  path?: string;
  exclude?: string;
  result?: any;
}

export const GrepWidget: React.FC<GrepWidgetProps> = ({ pattern, include, path, exclude, result }) => {
  const [isExpanded, setIsExpanded] = useState(true);
  
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
  
  // Parse grep results to extract file paths and matches
  const parseGrepResults = (content: string) => {
    const lines = content.split('\n').filter(line => line.trim());
    const results: Array<{
      file: string;
      lineNumber: number;
      content: string;
    }> = [];
    
    lines.forEach(line => {
      // Common grep output format: filename:lineNumber:content
      const match = line.match(/^(.+?):(\d+):(.*)$/);
      if (match) {
        results.push({
          file: match[1],
          lineNumber: parseInt(match[2], 10),
          content: match[3]
        });
      }
    });
    
    return results;
  };
  
  const grepResults = result && !isError ? parseGrepResults(resultContent) : [];
  
  // Group results by file
  const groupedResults = grepResults.reduce((acc, result) => {
    if (!acc[result.file]) {
      acc[result.file] = [];
    }
    acc[result.file].push(result);
    return acc;
  }, {} as Record<string, typeof grepResults>);
  
  return (
    <div className="space-y-2">
      <div className="flex items-center gap-2 p-3 rounded-lg bg-gradient-to-r from-emerald-500/10 to-teal-500/10 border border-emerald-500/20">
        <Search className="h-4 w-4 text-emerald-500" />
        <span className="text-sm font-medium">Searching with grep</span>
        {!result && (
          <div className="ml-auto flex items-center gap-1 text-xs text-muted-foreground">
            <div className="h-2 w-2 bg-emerald-500 rounded-full animate-pulse" />
            <span>Searching...</span>
          </div>
        )}
      </div>
      
      {/* Search Parameters */}
      <div className="rounded-lg border bg-muted/20 p-3 space-y-2">
        <div className="grid gap-2">
          {/* Pattern with regex highlighting */}
          <div className="flex items-start gap-3">
            <div className="flex items-center gap-1.5 min-w-[80px]">
              <Code className="h-3 w-3 text-emerald-500" />
              <span className="text-xs font-medium text-muted-foreground">Pattern</span>
            </div>
            <code className="flex-1 font-mono text-sm bg-emerald-500/10 border border-emerald-500/20 px-3 py-1.5 rounded-md text-emerald-600 dark:text-emerald-400">
              {pattern}
            </code>
          </div>
          
          {/* Path */}
          {path && (
            <div className="flex items-start gap-3">
              <div className="flex items-center gap-1.5 min-w-[80px]">
                <FolderOpen className="h-3 w-3 text-muted-foreground" />
                <span className="text-xs font-medium text-muted-foreground">Path</span>
              </div>
              <code className="flex-1 font-mono text-xs bg-muted px-2 py-1 rounded truncate">
                {path}
              </code>
            </div>
          )}
          
          {/* Include/Exclude patterns in a row */}
          {(include || exclude) && (
            <div className="flex gap-4">
              {include && (
                <div className="flex items-center gap-2 flex-1">
                  <div className="flex items-center gap-1.5">
                    <FilePlus className="h-3 w-3 text-green-500" />
                    <span className="text-xs font-medium text-muted-foreground">Include</span>
                  </div>
                  <code className="font-mono text-xs bg-green-500/10 border border-green-500/20 px-2 py-0.5 rounded text-green-600 dark:text-green-400">
                    {include}
                  </code>
                </div>
              )}
              
              {exclude && (
                <div className="flex items-center gap-2 flex-1">
                  <div className="flex items-center gap-1.5">
                    <X className="h-3 w-3 text-red-500" />
                    <span className="text-xs font-medium text-muted-foreground">Exclude</span>
                  </div>
                  <code className="font-mono text-xs bg-red-500/10 border border-red-500/20 px-2 py-0.5 rounded text-red-600 dark:text-red-400">
                    {exclude}
                  </code>
                </div>
              )}
            </div>
          )}
        </div>
      </div>
      
      {/* Results */}
      {result && (
        <div className="space-y-2">
          {isError ? (
            <div className="flex items-center gap-3 p-4 rounded-lg bg-red-500/10 border border-red-500/20">
              <AlertCircle className="h-5 w-5 text-red-500 flex-shrink-0" />
              <div className="text-sm text-red-600 dark:text-red-400">
                {resultContent || "Search failed"}
              </div>
            </div>
          ) : grepResults.length > 0 ? (
            <>
              <button
                onClick={() => setIsExpanded(!isExpanded)}
                className="flex items-center gap-2 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors"
              >
                {isExpanded ? (
                  <ChevronDown className="h-3.5 w-3.5" />
                ) : (
                  <ChevronRight className="h-3.5 w-3.5" />
                )}
                <span>{grepResults.length} matches found</span>
              </button>
              
              {isExpanded && (
                <div className="space-y-3">
                  {Object.entries(groupedResults).map(([file, matches]) => {
                    const language = getLanguageFromPath(file);
                    
                    return (
                      <div key={file} className="rounded-lg border bg-muted/10 overflow-hidden">
                        <div className="px-3 py-2 bg-muted/20 flex items-center gap-2">
                          <FileText className="h-3.5 w-3.5 text-muted-foreground" />
                          <span className="text-xs font-mono text-muted-foreground flex-1 truncate">
                            {file}
                          </span>
                          <Badge variant="secondary" className="text-xs">
                            {matches.length} {matches.length === 1 ? 'match' : 'matches'}
                          </Badge>
                        </div>
                        
                        <div className="divide-y divide-border/50">
                          {matches.map((match, idx) => (
                            <div key={idx} className="flex">
                              <div className="w-12 py-2 px-3 text-right text-xs text-muted-foreground bg-muted/10 border-r">
                                {match.lineNumber}
                              </div>
                              <div className="flex-1 py-2 px-3 overflow-x-auto">
                                <SyntaxHighlighter
                                  language={language}
                                  style={claudeSyntaxTheme}
                                  customStyle={{
                                    margin: 0,
                                    padding: 0,
                                    background: 'transparent',
                                    fontSize: '0.75rem',
                                    lineHeight: '1.5'
                                  }}
                                  PreTag="div"
                                  wrapLongLines={false}
                                >
                                  {match.content}
                                </SyntaxHighlighter>
                              </div>
                            </div>
                          ))}
                        </div>
                      </div>
                    );
                  })}
                </div>
              )}
            </>
          ) : (
            <div className="text-sm text-muted-foreground text-center py-8">
              No matches found
            </div>
          )}
        </div>
      )}
    </div>
  );
};