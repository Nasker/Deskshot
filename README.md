# deskshot

Snapshot the windows open on your Windows virtual desktops and restore them later.

```
deskshot capture layout.json   # record apps, positions, sizes, states, desktops
deskshot restore layout.json   # relaunch missing apps and put everything back
```

Or run `deskshot-gui` for a small window with Save and Restore buttons.

Requires Windows 11 24H2 or later (virtual desktop support comes from `winvd`).

## Layout

```
src/
  domain/      model + ports (traits). No platform code.
  app/         use cases: CaptureService, RestoreService, ExecutableMatcher
  infra/       adapters: JSON store, process launcher, Win32 + winvd
  composition.rs  wires adapters into use cases (capture / restore)
  main.rs, cli.rs CLI front-end
  bin/deskshot-gui.rs  GUI front-end (egui + native file dialogs)
```

Dependencies point inwards: `app` only knows the traits in `domain::ports`,
so it is unit-tested with in-memory fakes (`cargo test` runs on any OS).
All `unsafe` Win32 code is confined to `infra/windows/sys.rs`.

## Known limitations

- Snapped windows return to their exact on-screen rectangle, but Windows
  won't treat them as a snap group (no shared resizing) — there's no public API for that.
- Apps are relaunched with no arguments, so open documents and tabs are not restored.
- UWP / Store apps are skipped.
- Windows of elevated (admin) apps can't be moved unless deskshot also runs elevated.
- `winvd` wraps undocumented COM interfaces; after a big Windows update, update the crate.
