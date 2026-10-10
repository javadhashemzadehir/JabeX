"use client";

import { CommandInput } from "@/components/ui/command";

export function SearchInput({
  value,
  onValueChange,
  placeholder,
}: {
  value: string;
  onValueChange: (value: string) => void;
  placeholder: string;
}) {
  return (
    <CommandInput
      value={value}
      onValueChange={onValueChange}
      placeholder={placeholder}
      dir="auto"
    />
  );
}
