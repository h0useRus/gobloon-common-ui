# Gobloon common UI

The egui widget kit and the localization loader shared by the Gobloon apps
([Joystick Gobloon](https://github.com/h0useRus/JoystickGobloon) and Keyboard Gobloon).

- **gob-ui**: one look for every control. Palette and text sizes (`theme`), sizes and spacing on a
  4 px grid (`tokens`), and the widgets built on them: icon and toolbar buttons, tabs, toggle
  switches, ticks, a numeric up-down with slider, text fields, editable text, modal dialogs and
  Windows-style dialog windows. Icons are Phosphor's regular set. Nothing in it knows about an app:
  colors come in as a `Palettes`, texts as arguments.
- **gob-i18n**: Fluent localization. The app passes its embedded languages to `I18n::new`, English
  is the fallback, and extra `<code>.ftl` files load from a folder. `check_complete` lets an app's
  test check that every language has every English key.

## Using it

```toml
[workspace.dependencies]
gob-ui = { git = "https://github.com/h0useRus/gobloon-common-ui", tag = "v0.1.0" }
gob-i18n = { git = "https://github.com/h0useRus/gobloon-common-ui", tag = "v0.1.0" }
```

Both apps must use the same egui version as this repo. To try a change in an app before it is
tagged, clone this repo next to the app and add a gitignored `.cargo/config.toml` there:

```toml
[patch."https://github.com/h0useRus/gobloon-common-ui"]
gob-ui = { path = "../gobloon-common-ui/crates/gob-ui" }
gob-i18n = { path = "../gobloon-common-ui/crates/gob-i18n" }
```

`cargo run -p gob-ui --example gallery` shows every widget, in light and dark.

## Rules

- A change to a widget's look goes here, never into an app.
- Nothing app-specific: no product names, strings, palettes or fonts.
- Conventional Commits; versions are tags `vX.Y.Z` (`feat` -> minor, `fix` -> patch, breaking -> major,
  minor while 0.x). Apps move to a new tag on purpose.
- egui upgrades happen here first; the apps move to the new tag together.

## License

MIT
