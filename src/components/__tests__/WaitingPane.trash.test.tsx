import { act, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { WaitingPane } from "../WaitingPane";
import { useStore } from "../../store";
import "../../i18n";
import type { Snapshot, User } from "../../types";

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

const snap: Snapshot = {
  type: "state",
  playing: [],
  waiting: [mkUser("wait-1", "Waiting One")],
  waitingTotal: 1,
  trash: [mkUser("trash-1", "Deleted One")],
  enabled: true,
  language: "en",
  maxWaiting: 3,
  theme: "midnight",
};

describe("WaitingPane trash section", () => {
  beforeEach(() => {
    invokeSpy.mockClear();
    act(() => {
      useStore.setState({ snap });
    });
  });

  afterEach(() => {
    act(() => {
      useStore.setState({ snap: null });
    });
  });

  it("renders trash below waiting and restores the selected user", () => {
    render(<WaitingPane />);

    const waitingUser = screen.getByText("Waiting One");
    const deletedUser = screen.getByText("Deleted One");

    expect(
      waitingUser.compareDocumentPosition(deletedUser) & Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();

    fireEvent.click(screen.getByRole("button", { name: /Deleted One/ }));

    expect(invokeSpy).toHaveBeenCalledWith("restore_user", { userId: "trash-1" });
  });
});
