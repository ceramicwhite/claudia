import React from "react";
import { AlertCircle } from "lucide-react";
import { Alert, AlertDescription } from "@/components/ui/alert";

interface WidgetErrorProps {
  error: string;
  title?: string;
}

export const WidgetError: React.FC<WidgetErrorProps> = ({ error, title = "Error" }) => {
  return (
    <Alert variant="destructive">
      <AlertCircle className="h-4 w-4" />
      <AlertDescription className="font-mono text-xs">
        {title}: {error}
      </AlertDescription>
    </Alert>
  );
};