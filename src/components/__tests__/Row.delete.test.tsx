import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { Row } from "../Row";
import type { User } from "../../types";
import "../../i18n";

const invokeSpy = vi.fn<(...args: unknown[]) => Promise<unknown>>(async () => null);
(globalThis as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__ = {
  invoke: (cmd: string, args: unknown) => invokeSpy(cmd, args),
  transformCallback: () => 0,
  unregisterListener: () => {},
};

const mkUser = (id: string, displayName: string): User => ({
  id,
  name: id,
  displayName,
  joinCount: 0,
  lastJoinAt: null,
  enqueuedAt: 0,
  manualOrder: null,
  firstTimeToday: true,
});

describe("Row trash button", () => {
  beforeEach(() => invokeSpy.mockClear());

  it("playing clear button uses the non-trash clear command", () => {
    render(<Row user={mkUser("u-alpha", "Alpha")} zone="playing" index={0} onDragStart={() => () => {}} />);

    const buttons = screen.getAllByRole("button");
    expect(buttons).toHaveLength(2);

    fireEvent.click(buttons[0]);
    expect(invokeSpy).toHaveBeenCalledWith("clear_playing_user", { userId: "u-alpha" });
  });

  it("clicking trash on row A sends A's id, not another row's", () => {
    const users = [
      mkUser("u-alpha", "Alpha"),
      mkUser("u-beta", "Beta"),
      mkUser("u-gamma", "Gamma"),
    ];
    render(
      <>
        {users.map((u, i) => (
          <Row key={u.id} user={u} zone="waiting" index={i} onDragStart={() => () => {}} />
        ))}
      </>
    );
    const buttons = screen.getAllByRole("button");
    expect(buttons).toHaveLength(3);

    fireEvent.click(buttons[1]);
    expect(invokeSpy).toHaveBeenCalledTimes(1);
    expect(invokeSpy).toHaveBeenCalledWith("trash_user", { userId: "u-beta" });

    fireEvent.click(buttons[0]);
    expect(invokeSpy).toHaveBeenLastCalledWith("trash_user", { userId: "u-alpha" });

    fireEvent.click(buttons[2]);
    expect(invokeSpy).toHaveBeenLastCalledWith("trash_user", { userId: "u-gamma" });
  });

  it("after re-render with reordered users, click still targets the visually matching id", () => {
    const first = [mkUser("u1", "One"), mkUser("u2", "Two")];
    const { rerender } = render(
      <>
        {first.map((u, i) => (
          <Row key={u.id} user={u} zone="waiting" index={i} onDragStart={() => () => {}} />
        ))}
      </>
    );
    // Reorder (swap positions)
    const reordered = [first[1], first[0]];
    rerender(
      <>
        {reordered.map((u, i) => (
          <Row key={u.id} user={u} zone="waiting" index={i} onDragStart={() => () => {}} />
        ))}
      </>
    );
    // First button in DOM now belongs to u2
    const btns = screen.getAllByRole("button");
    fireEvent.click(btns[0]);
    expect(invokeSpy).toHaveBeenLastCalledWith("trash_user", { userId: "u2" });
  });
});
