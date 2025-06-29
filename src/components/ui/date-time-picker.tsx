import React, { useEffect, useState } from "react";
import { Calendar as CalendarIcon, Clock } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import * as PopoverPrimitive from "@radix-ui/react-popover";
import { Calendar } from "@/components/ui/calendar";
import { TimePicker } from "@/components/ui/time-picker";
import { cn } from "@/lib/utils";

interface DateTimePickerProps {
  /**
   * The selected datetime value (ISO 8601 string)
   */
  value?: string;
  /**
   * Callback when datetime changes
   */
  onChange: (value: string | undefined) => void;
  /**
   * Optional placeholder text
   */
  placeholder?: string;
  /**
   * Optional className
   */
  className?: string;
  /**
   * Optional disabled state
   */
  disabled?: boolean;
  /**
   * Minimum datetime (ISO 8601 string)
   */
  min?: string;
}

/**
 * DateTimePicker component for selecting date and time
 * 
 * @example
 * <DateTimePicker
 *   value={scheduledTime}
 *   onChange={setScheduledTime}
 *   placeholder="Select date and time"
 * />
 */
export const DateTimePicker: React.FC<DateTimePickerProps> = ({
  value,
  onChange,
  placeholder = "Select date and time",
  className,
  disabled = false,
  min,
}) => {
  const [isOpen, setIsOpen] = useState(false);
  const [selectedDate, setSelectedDate] = useState<Date | undefined>(
    value ? new Date(value) : undefined
  );
  const [selectedTime, setSelectedTime] = useState<string>(() => {
    if (value) {
      const date = new Date(value);
      return `${String(date.getHours()).padStart(2, "0")}:${String(date.getMinutes()).padStart(2, "0")}`;
    }
    return "";
  });

  useEffect(() => {
    if (value) {
      const date = new Date(value);
      setSelectedDate(date);
      setSelectedTime(`${String(date.getHours()).padStart(2, "0")}:${String(date.getMinutes()).padStart(2, "0")}`);
    } else {
      setSelectedDate(undefined);
      setSelectedTime("");
    }
  }, [value]);

  // Format datetime for display
  const formatDateTime = (isoString?: string) => {
    if (!isoString) return "";
    const date = new Date(isoString);
    const options: Intl.DateTimeFormatOptions = {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
      hour12: true
    };
    return date.toLocaleString(undefined, options);
  };

  const handleDateSelect = (date: Date) => {
    setSelectedDate(date);
    
    // If we have a time, combine it with the new date
    if (selectedTime) {
      const [hours, minutes] = selectedTime.split(':').map(Number);
      const newDateTime = new Date(date);
      newDateTime.setHours(hours, minutes, 0, 0);
      onChange(newDateTime.toISOString());
    } else {
      // Set time to current time if no time is selected
      const now = new Date();
      const newDateTime = new Date(date);
      newDateTime.setHours(now.getHours(), now.getMinutes(), 0, 0);
      const timeString = `${String(now.getHours()).padStart(2, "0")}:${String(now.getMinutes()).padStart(2, "0")}`;
      setSelectedTime(timeString);
      onChange(newDateTime.toISOString());
    }
  };

  const handleTimeChange = (time: string) => {
    setSelectedTime(time);
    
    if (selectedDate) {
      const [hours, minutes] = time.split(':').map(Number);
      const newDateTime = new Date(selectedDate);
      newDateTime.setHours(hours, minutes, 0, 0);
      onChange(newDateTime.toISOString());
    } else {
      // If no date is selected, use today
      const today = new Date();
      const [hours, minutes] = time.split(':').map(Number);
      today.setHours(hours, minutes, 0, 0);
      setSelectedDate(today);
      onChange(today.toISOString());
    }
  };

  const handleSave = () => {
    if (selectedDate && selectedTime) {
      const [hours, minutes] = selectedTime.split(':').map(Number);
      const newDateTime = new Date(selectedDate);
      newDateTime.setHours(hours, minutes, 0, 0);
      onChange(newDateTime.toISOString());
    } else if (selectedDate) {
      // If only date is selected, use current time
      const now = new Date();
      const newDateTime = new Date(selectedDate);
      newDateTime.setHours(now.getHours(), now.getMinutes(), 0, 0);
      onChange(newDateTime.toISOString());
    }
    setIsOpen(false);
  };

  const handleClear = () => {
    setSelectedDate(undefined);
    setSelectedTime("");
    onChange(undefined);
    setIsOpen(false);
  };

  const isDateDisabled = (date: Date) => {
    if (!min) return false;
    const minDate = new Date(min);
    return date < minDate;
  };

  return (
    <PopoverPrimitive.Root open={isOpen} onOpenChange={setIsOpen}>
      <PopoverPrimitive.Trigger asChild>
        <Button
          variant="outline"
          className={cn(
            "justify-start text-left font-normal",
            !value && "text-muted-foreground",
            className
          )}
          disabled={disabled}
        >
          <CalendarIcon className="mr-2 h-4 w-4" />
          {value ? formatDateTime(value) : placeholder}
        </Button>
      </PopoverPrimitive.Trigger>
      
      <PopoverPrimitive.Portal>
        <PopoverPrimitive.Content
          className={cn(
            "z-50 w-auto rounded-md border border-border bg-popover p-0 text-popover-foreground shadow-md",
            "data-[state=open]:animate-in data-[state=closed]:animate-out",
            "data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0",
            "data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95",
            "data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2",
            "data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2"
          )}
          align="start"
          sideOffset={4}
        >
          <div className="flex">
            {/* Calendar */}
            <Calendar
              selected={selectedDate}
              onSelect={handleDateSelect}
              disabled={isDateDisabled}
              className="rounded-md border-r border-border"
            />
            
            {/* Time Picker */}
            <div className="p-3 space-y-4">
              <div className="space-y-2">
                <Label className="text-sm font-medium">Time</Label>
                <TimePicker
                  value={selectedTime}
                  onChange={handleTimeChange}
                  disabled={disabled}
                />
              </div>
              
              <div className="flex flex-col space-y-2">
                <Button
                  onClick={handleSave}
                  size="sm"
                  className="w-full"
                  disabled={!selectedDate}
                >
                  Save
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleClear}
                  className="w-full"
                  disabled={!value}
                >
                  Clear
                </Button>
              </div>
              
              {value && (
                <div className="text-xs text-muted-foreground border-t border-border pt-2">
                  <p>Selected: {formatDateTime(value)}</p>
                  <p className="mt-1">Timezone: {Intl.DateTimeFormat().resolvedOptions().timeZone}</p>
                </div>
              )}
            </div>
          </div>
        </PopoverPrimitive.Content>
      </PopoverPrimitive.Portal>
    </PopoverPrimitive.Root>
  );
};