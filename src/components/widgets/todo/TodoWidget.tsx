import React from "react";
import { FileEdit } from "lucide-react";
import { WidgetHeader } from "../common/WidgetHeader";
import { TodoItem } from "./TodoItem";

interface TodoWidgetProps {
  todos: any[];
  result?: any;
}

export const TodoWidget: React.FC<TodoWidgetProps> = ({ todos, result: _result }) => {
  return (
    <div className="space-y-2">
      <WidgetHeader icon={FileEdit} title="Todo List" className="mb-3" />
      <div className="space-y-2">
        {todos.map((todo, idx) => (
          <TodoItem key={todo.id || idx} todo={todo} />
        ))}
      </div>
    </div>
  );
};