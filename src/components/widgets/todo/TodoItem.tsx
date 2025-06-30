import React from "react";
import { CheckCircle2, Circle, Clock } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";
import { TodoStatus, Priority } from "@/constants";

interface TodoItemProps {
  todo: {
    id?: string;
    content: string;
    status: string;
    priority?: string;
  };
}

const statusIcons = {
  [TodoStatus.COMPLETED]: <CheckCircle2 className="h-4 w-4 text-green-500" />,
  [TodoStatus.IN_PROGRESS]: <Clock className="h-4 w-4 text-blue-500 animate-pulse" />,
  [TodoStatus.PENDING]: <Circle className="h-4 w-4 text-muted-foreground" />
};

const priorityColors = {
  [Priority.HIGH]: "bg-red-500/10 text-red-500 border-red-500/20",
  [Priority.MEDIUM]: "bg-yellow-500/10 text-yellow-500 border-yellow-500/20",
  [Priority.LOW]: "bg-green-500/10 text-green-500 border-green-500/20"
};

export const TodoItem: React.FC<TodoItemProps> = ({ todo }) => {
  return (
    <div
      className={cn(
        "flex items-start gap-3 p-3 rounded-lg border bg-card/50",
        todo.status === TodoStatus.COMPLETED && "opacity-60"
      )}
    >
      <div className="mt-0.5">
        {statusIcons[todo.status as keyof typeof statusIcons] || statusIcons[TodoStatus.PENDING]}
      </div>
      <div className="flex-1 space-y-1">
        <p className={cn(
          "text-sm",
          todo.status === TodoStatus.COMPLETED && "line-through"
        )}>
          {todo.content}
        </p>
        {todo.priority && (
          <Badge 
            variant="outline" 
            className={cn(
              "text-xs",
              priorityColors[todo.priority as keyof typeof priorityColors]
            )}
          >
            {todo.priority}
          </Badge>
        )}
      </div>
    </div>
  );
};