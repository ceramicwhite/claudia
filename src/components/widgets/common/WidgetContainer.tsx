import React, { useState } from "react";
import { ChevronDown, ChevronUp, Maximize2, X, type LucideIcon } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { cn } from "@/lib/utils";
import { createPortal } from "react-dom";

interface WidgetContainerProps {
  icon: LucideIcon;
  title: string;
  children: React.ReactNode;
  className?: string;
  defaultExpanded?: boolean;
  headerContent?: React.ReactNode;
  collapsible?: boolean;
  expandable?: boolean;
}

export const WidgetContainer: React.FC<WidgetContainerProps> = ({
  icon: Icon,
  title,
  children,
  className,
  defaultExpanded = true,
  headerContent,
  collapsible = true,
  expandable = true,
}) => {
  const [isExpanded, setIsExpanded] = useState(defaultExpanded);
  const [isFullscreen, setIsFullscreen] = useState(false);

  const content = (
    <Card className={cn("overflow-hidden", className, isFullscreen && "h-full")}>
      <CardContent className="p-4">
        <div className="flex items-center justify-between mb-3">
          <div className="flex items-center gap-2">
            <Icon className="h-4 w-4 text-primary" />
            <span className="text-sm font-medium">{title}</span>
            {headerContent}
          </div>
          <div className="flex items-center gap-1">
            {expandable && !isFullscreen && (
              <Button
                variant="ghost"
                size="icon"
                className="h-6 w-6"
                onClick={() => setIsFullscreen(true)}
              >
                <Maximize2 className="h-3 w-3" />
              </Button>
            )}
            {isFullscreen && (
              <Button
                variant="ghost"
                size="icon"
                className="h-6 w-6"
                onClick={() => setIsFullscreen(false)}
              >
                <X className="h-3 w-3" />
              </Button>
            )}
            {collapsible && !isFullscreen && (
              <Button
                variant="ghost"
                size="icon"
                className="h-6 w-6"
                onClick={() => setIsExpanded(!isExpanded)}
              >
                {isExpanded ? (
                  <ChevronUp className="h-3 w-3" />
                ) : (
                  <ChevronDown className="h-3 w-3" />
                )}
              </Button>
            )}
          </div>
        </div>
        {isExpanded && children}
      </CardContent>
    </Card>
  );

  if (isFullscreen) {
    return createPortal(
      <div className="fixed inset-0 z-50 bg-background/95 backdrop-blur-sm">
        <div className="w-full h-full p-4">
          {content}
        </div>
      </div>,
      document.body
    );
  }

  return content;
};