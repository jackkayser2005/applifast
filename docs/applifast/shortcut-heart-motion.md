# Favorite shortcut heart feedback

Clicking an Apple favorite control started the shared heart pop before queuing
the optimistic favorite action. The **B** shortcut queued the action without
starting that animation. Both the main-player and matching-row filled hearts
therefore stayed at their resting size after the shortcut.

The Apple shortcut now starts the same URI-scoped animation before sending its
existing `SetSavedMany` action. The action, pending-write behavior, account
handling, read-only filled-heart semantics and keyboard focus/dialog rules stay
the same. The shared motion helper still honors Reduce motion and cancels an
interrupted pop without reviving it when the setting is turned off again.

The existing actual-interface regression now exercises three input routes:
row control, player control and a real egui B key event. It inspects both filled
hearts' painted image bounds in light/dark themes and at 760/1100 widths, checks
the enlarged intermediate state, then verifies settling and Reduce motion
interruption/no revival. Before the fix the shortcut case fails on a fixed
16-pixel heart. After the one-line fix all twelve scenarios pass. The test uses
isolated sample data and sends no real Apple write.

This restores an interaction within the previously requested PR #13 heart-pop
scope. There is no layout, endpoint, credential, storage-format or dependency
change. Existing [resting native comparisons](review-integration/index.html) do
not show this short transition. Native keyboard/transition acceptance remains
separate from the deterministic painted-frame regression; the current
[native frame evidence](native-motion-frames.md) proves playing indicators only.
