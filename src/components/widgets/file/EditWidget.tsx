import React from "react";
import { FileEdit } from "lucide-react";
import { cn } from "@/lib/utils";
import { Prism as SyntaxHighlighter } from "react-syntax-highlighter";
import { claudeSyntaxTheme } from "@/lib/claudeSyntaxTheme";
import * as Diff from 'diff';
import { getLanguageFromPath } from "./utils";

interface EditWidgetProps {
  file_path: string;
  old_string: string;
  new_string: string;
  result?: any;
}

const EditWidgetComponent: React.FC<EditWidgetProps> = ({ 
  file_path, 
  old_string, 
  new_string, 
  result: _result 
}) => {
  const diffResult = Diff.diffLines(old_string || '', new_string || '', { 
    newlineIsToken: true,
    ignoreWhitespace: false 
  });
  const language = getLanguageFromPath(file_path);

  return (
    <div className="space-y-2">
      <div className="flex items-center gap-2 mb-2">
        <FileEdit className="h-4 w-4 text-primary" />
        <span className="text-sm font-medium">Applying Edit to:</span>
        <code className="text-sm font-mono bg-background px-2 py-0.5 rounded flex-1 truncate">
          {file_path}
        </code>
      </div>

      <div className="rounded-lg border bg-zinc-950 overflow-hidden text-xs font-mono">
        <div className="max-h-[440px] overflow-y-auto overflow-x-auto">
          {diffResult.map((part, index) => {
            const partClass = part.added 
              ? 'bg-green-950/20' 
              : part.removed 
              ? 'bg-red-950/20'
              : '';
            
            if (!part.added && !part.removed && part.count && part.count > 8) {
              return (
                <div key={index} className="px-4 py-1 bg-zinc-900 border-y border-zinc-800 text-center text-zinc-500 text-xs">
                  ... {part.count} unchanged lines ...
                </div>
              );
            }
            
            const value = part.value.endsWith('\n') ? part.value.slice(0, -1) : part.value;

            return (
              <div key={index} className={cn(partClass, "flex")}>
                <div className="w-8 select-none text-center flex-shrink-0">
                  {part.added ? <span className="text-green-400">+</span> : part.removed ? <span className="text-red-400">-</span> : null}
                </div>
                <div className="flex-1">
                  <SyntaxHighlighter
                    language={language}
                    style={claudeSyntaxTheme}
                    PreTag="div"
                    wrapLongLines={false}
                    customStyle={{
                      margin: 0,
                      padding: 0,
                      background: 'transparent',
                    }}
                    codeTagProps={{
                      style: {
                        fontSize: '0.75rem',
                        lineHeight: '1.6',
                      }
                    }}
                  >
                    {value}
                  </SyntaxHighlighter>
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
};

export const EditWidget = React.memo(EditWidgetComponent);