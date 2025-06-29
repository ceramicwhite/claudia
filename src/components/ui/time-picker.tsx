import React, { useState } from "react";
import { Clock, ChevronUp, ChevronDown } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";

interface TimePickerProps {
  value?: string; // HH:MM format
  onChange: (value: string) => void;
  disabled?: boolean;
  className?: string;
}

export const TimePicker: React.FC<TimePickerProps> = ({
  value,
  onChange,
  disabled,
  className,
}) => {
  const [hour, minute] = value ? value.split(":").map(Number) : [new Date().getHours(), new Date().getMinutes()];
  const [hourInput, setHourInput] = useState(String(hour).padStart(2, "0"));
  const [minuteInput, setMinuteInput] = useState(String(minute).padStart(2, "0"));

  const updateTime = (newHour: number, newMinute: number) => {
    const validHour = Math.max(0, Math.min(23, newHour));
    const validMinute = Math.max(0, Math.min(59, newMinute));
    const timeString = `${String(validHour).padStart(2, "0")}:${String(validMinute).padStart(2, "0")}`;
    onChange(timeString);
  };

  const handleHourChange = (newHour: number) => {
    const validHour = Math.max(0, Math.min(23, newHour));
    setHourInput(String(validHour).padStart(2, "0"));
    updateTime(validHour, minute);
  };

  const handleMinuteChange = (newMinute: number) => {
    const validMinute = Math.max(0, Math.min(59, newMinute));
    setMinuteInput(String(validMinute).padStart(2, "0"));
    updateTime(hour, validMinute);
  };

  const handleHourInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;
    setHourInput(value);
    
    const numValue = parseInt(value, 10);
    if (!isNaN(numValue) && numValue >= 0 && numValue <= 23) {
      updateTime(numValue, minute);
    }
  };

  const handleMinuteInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;
    setMinuteInput(value);
    
    const numValue = parseInt(value, 10);
    if (!isNaN(numValue) && numValue >= 0 && numValue <= 59) {
      updateTime(hour, numValue);
    }
  };

  const handleHourInputBlur = () => {
    const numValue = parseInt(hourInput, 10);
    if (isNaN(numValue) || numValue < 0 || numValue > 23) {
      setHourInput(String(hour).padStart(2, "0"));
    }
  };

  const handleMinuteInputBlur = () => {
    const numValue = parseInt(minuteInput, 10);
    if (isNaN(numValue) || numValue < 0 || numValue > 59) {
      setMinuteInput(String(minute).padStart(2, "0"));
    }
  };

  return (
    <div className={cn("flex items-center space-x-2", className)}>
      <Clock className="h-4 w-4 text-muted-foreground" />
      
      {/* Hour input */}
      <div className="flex flex-col items-center">
        <Button
          variant="ghost"
          size="sm"
          className="h-6 w-8 p-0"
          onClick={() => handleHourChange(hour + 1)}
          disabled={disabled}
        >
          <ChevronUp className="h-3 w-3" />
        </Button>
        <Input
          value={hourInput}
          onChange={handleHourInputChange}
          onBlur={handleHourInputBlur}
          className="h-8 w-12 text-center text-sm p-1"
          disabled={disabled}
          maxLength={2}
        />
        <Button
          variant="ghost"
          size="sm"
          className="h-6 w-8 p-0"
          onClick={() => handleHourChange(hour - 1)}
          disabled={disabled}
        >
          <ChevronDown className="h-3 w-3" />
        </Button>
      </div>

      <span className="text-muted-foreground">:</span>

      {/* Minute input */}
      <div className="flex flex-col items-center">
        <Button
          variant="ghost"
          size="sm"
          className="h-6 w-8 p-0"
          onClick={() => handleMinuteChange(minute + 1)}
          disabled={disabled}
        >
          <ChevronUp className="h-3 w-3" />
        </Button>
        <Input
          value={minuteInput}
          onChange={handleMinuteInputChange}
          onBlur={handleMinuteInputBlur}
          className="h-8 w-12 text-center text-sm p-1"
          disabled={disabled}
          maxLength={2}
        />
        <Button
          variant="ghost"
          size="sm"
          className="h-6 w-8 p-0"
          onClick={() => handleMinuteChange(minute - 1)}
          disabled={disabled}
        >
          <ChevronDown className="h-3 w-3" />
        </Button>
      </div>
    </div>
  );
};
