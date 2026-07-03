import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter, Routes, Route } from "react-router-dom";
import { Home } from "lucide-react";
import { SidebarNavItem } from "./SidebarNavItem";

function renderWithRouter(
  initialEntries: string[],
  ui: React.ReactElement
) {
  return render(
    <MemoryRouter initialEntries={initialEntries}>
      <Routes>
        <Route path="*" element={ui} />
      </Routes>
    </MemoryRouter>
  );
}

describe("SidebarNavItem", () => {
  it("renders label and icon", () => {
    renderWithRouter(["/"], <SidebarNavItem to="/" label="Home" icon={Home} />);
    expect(screen.getByText("Home")).toBeInTheDocument();
    expect(document.querySelector("svg")).toBeInTheDocument();
  });

  it("calls onClick when clicked", async () => {
    const onClick = vi.fn();
    renderWithRouter(
      ["/"],
      <SidebarNavItem to="/dashboard" label="Dashboard" icon={Home} onClick={onClick} />
    );
    await userEvent.click(screen.getByRole("link", { name: "Dashboard" }));
    expect(onClick).toHaveBeenCalledOnce();
  });

  it("marks active link when route matches", () => {
    renderWithRouter(
      ["/dashboard"],
      <SidebarNavItem to="/dashboard" label="Dashboard" icon={Home} />
    );
    const link = screen.getByRole("link", { name: "Dashboard" });
    expect(link).toHaveAttribute("aria-current", "page");
  });
});
