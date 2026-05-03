import { render, screen } from "@testing-library/react";
import { Row } from "../Row";
import { describe, it, expect } from "vitest";
import "../../i18n";

const user = {
  id: "u1", name: "alice", displayName: "Alice",
  joinCount: 0, lastJoinAt: null, enqueuedAt: 0, manualOrder: null,
  firstTimeToday: true,
};

describe("Row", () => {
  it("shows badge when firstTimeToday is true", () => {
    render(<Row user={user} zone="playing" index={0} onDragStart={() => () => {}} />);
    expect(screen.getByText("初")).toBeInTheDocument();
  });

  it("keeps the full display name available when the row truncates", () => {
    const longName = "this_is_a_very_long_display_name_that_should_be_truncated_in_the_row";
    render(<Row user={{ ...user, displayName: longName }} zone="playing" index={0} onDragStart={() => () => {}} />);
    expect(screen.getByText(longName)).toHaveAttribute("title", longName);
  });

  it("hides badge when firstTimeToday is false", () => {
    const repeat = { ...user, firstTimeToday: false, lastJoinAt: Date.now() - 60_000 };
    render(<Row user={repeat} zone="playing" index={0} onDragStart={() => () => {}} />);
    expect(screen.queryByText("初")).not.toBeInTheDocument();
  });

  it("badge persists after promotion (lastJoinAt now, firstTimeToday still true)", () => {
    const promoted = { ...user, lastJoinAt: Date.now(), firstTimeToday: true };
    render(<Row user={promoted} zone="playing" index={0} onDragStart={() => () => {}} />);
    expect(screen.getByText("初")).toBeInTheDocument();
  });
});
