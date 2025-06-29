import React from "react";
import { ChevronLeft, ChevronRight } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

interface CalendarProps {
  selected?: Date;
  onSelect: (date: Date) => void;
  disabled?: (date: Date) => boolean;
  className?: string;
}

const MONTHS = [
  "January", "February", "March", "April", "May", "June",
  "July", "August", "September", "October", "November", "December"
];

const DAYS = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];

export const Calendar: React.FC<CalendarProps> = ({
  selected,
  onSelect,
  disabled,
  className,
}) => {
  const [currentMonth, setCurrentMonth] = React.useState(() => {
    return selected ? new Date(selected.getFullYear(), selected.getMonth(), 1) : new Date();
  });

  const today = new Date();
  const year = currentMonth.getFullYear();
  const month = currentMonth.getMonth();

  // Get first day of month and number of days
  const firstDayOfMonth = new Date(year, month, 1);
  const lastDayOfMonth = new Date(year, month + 1, 0);
  const firstDayWeekday = firstDayOfMonth.getDay();
  const daysInMonth = lastDayOfMonth.getDate();

  // Get days from previous month to fill the grid
  const daysFromPrevMonth = firstDayWeekday;
  const prevMonth = new Date(year, month - 1, 0);
  const daysInPrevMonth = prevMonth.getDate();

  // Calculate total cells needed
  const totalCells = Math.ceil((daysFromPrevMonth + daysInMonth) / 7) * 7;
  const daysFromNextMonth = totalCells - daysFromPrevMonth - daysInMonth;

  const handlePrevMonth = () => {
    setCurrentMonth(new Date(year, month - 1, 1));
  };

  const handleNextMonth = () => {
    setCurrentMonth(new Date(year, month + 1, 1));
  };

  const handleDateClick = (date: Date) => {
    if (!disabled || !disabled(date)) {
      onSelect(date);
    }
  };

  const isSelected = (date: Date) => {
    return selected && 
           date.getDate() === selected.getDate() &&
           date.getMonth() === selected.getMonth() &&
           date.getFullYear() === selected.getFullYear();
  };

  const isToday = (date: Date) => {
    return date.getDate() === today.getDate() &&
           date.getMonth() === today.getMonth() &&
           date.getFullYear() === today.getFullYear();
  };

  const isDisabled = (date: Date) => {
    return disabled ? disabled(date) : false;
  };

  const renderCalendarDays = () => {
    const days = [];

    // Previous month days
    for (let i = daysInPrevMonth - daysFromPrevMonth + 1; i <= daysInPrevMonth; i++) {
      const date = new Date(year, month - 1, i);
      days.push(
        <button
          key={`prev-${i}`}
          onClick={() => handleDateClick(date)}
          disabled={isDisabled(date)}
          className={cn(
            "h-8 w-8 text-sm font-normal text-muted-foreground hover:bg-accent hover:text-accent-foreground rounded-md transition-colors",
            "disabled:opacity-50 disabled:cursor-not-allowed"
          )}
        >
          {i}
        </button>
      );
    }

    // Current month days
    for (let i = 1; i <= daysInMonth; i++) {
      const date = new Date(year, month, i);
      const selected = isSelected(date);
      const today = isToday(date);
      const disabled = isDisabled(date);

      days.push(
        <button
          key={`current-${i}`}
          onClick={() => handleDateClick(date)}
          disabled={disabled}
          className={cn(
            "h-8 w-8 text-sm font-normal rounded-md transition-colors",
            "hover:bg-accent hover:text-accent-foreground",
            "disabled:opacity-50 disabled:cursor-not-allowed",
            selected && "bg-primary text-primary-foreground hover:bg-primary hover:text-primary-foreground",
            today && !selected && "bg-accent text-accent-foreground",
            !selected && !today && "text-foreground"
          )}
        >
          {i}
        </button>
      );
    }

    // Next month days
    for (let i = 1; i <= daysFromNextMonth; i++) {
      const date = new Date(year, month + 1, i);
      days.push(
        <button
          key={`next-${i}`}
          onClick={() => handleDateClick(date)}
          disabled={isDisabled(date)}
          className={cn(
            "h-8 w-8 text-sm font-normal text-muted-foreground hover:bg-accent hover:text-accent-foreground rounded-md transition-colors",
            "disabled:opacity-50 disabled:cursor-not-allowed"
          )}
        >
          {i}
        </button>
      );
    }

    return days;
  };

  return (
    <div className={cn("p-3", className)}>
      {/* Header */}
      <div className="flex items-center justify-between mb-4">
        <Button
          variant="outline"
          size="sm"
          onClick={handlePrevMonth}
          className="h-7 w-7 p-0"
        >
          <ChevronLeft className="h-4 w-4" />
        </Button>
        
        <div className="text-sm font-medium">
          {MONTHS[month]} {year}
        </div>
        
        <Button
          variant="outline"
          size="sm"
          onClick={handleNextMonth}
          className="h-7 w-7 p-0"
        >
          <ChevronRight className="h-4 w-4" />
        </Button>
      </div>

      {/* Days of week header */}
      <div className="grid grid-cols-7 gap-1 mb-2">
        {DAYS.map((day) => (
          <div
            key={day}
            className="h-8 w-8 text-xs font-medium text-muted-foreground flex items-center justify-center"
          >
            {day}
          </div>
        ))}
      </div>

      {/* Calendar grid */}
      <div className="grid grid-cols-7 gap-1">
        {renderCalendarDays()}
      </div>
    </div>
  );
};
