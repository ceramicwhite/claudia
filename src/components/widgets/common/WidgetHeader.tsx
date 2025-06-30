import React from "react";
import { type LucideIcon } from "lucide-react";
import { cn } from "@/lib/utils";

interface WidgetHeaderProps {
  icon: LucideIcon;
  title: string;
  className?: string;
  children?: React.ReactNode;
}

export const WidgetHeader: React.FC<WidgetHeaderProps> = ({
  icon: Icon,
  title,
  className,
  children,
}) => {
  return (
    <div className={cn("flex items-center gap-2", className)}>
      <Icon className="h-4 w-4 text-primary" />
      <span className="text-sm font-medium">{title}</span>
      {children}
    </div>
  );
};