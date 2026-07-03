import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import { Inbox } from "lucide-react";
import { EmptyState } from "./EmptyState";

describe("EmptyState", () => {
  it("renders title and description", () => {
    render(<EmptyState title="No data" description="Add a symbol to start." />);
    expect(screen.getByText("No data")).toBeInTheDocument();
    expect(screen.getByText("Add a symbol to start.")).toBeInTheDocument();
  });

  it("renders icon when provided", () => {
    render(<EmptyState title="Empty" icon={Inbox} />);
    expect(document.querySelector("svg")).toBeInTheDocument();
  });

  it("renders action slot", () => {
    render(<EmptyState title="Empty" action={<button type="button">Retry</button>} />);
    expect(screen.getByRole("button", { name: "Retry" })).toBeInTheDocument();
  });
});
